#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_wrap)]

//!# Unofficial high-level safe Rust bindings to ecCodes library
//!
//!This crate contains safe high-level bindings for ecCodes library.
//!Bindings can be considered safe mainly because all crate structures
//!will take ownership of the data in memory before passing the raw pointer to ecCodes.
//!**Currently only reading of GRIB files is supported.**
//!
//!Because of the ecCodes library API characteristics theses bindings are
//!rather thick wrapper to make this crate safe and convenient to use.
//!
//!This crate officially supports mainly Linux platforms as the ecCodes library supports them.
//!But it is possible to install ecCodes on MacOS and this crate successfully compiles and all tests pass.
//!
//!If you want to see more features released quicker do not hesitate
//!to contribute and check out [Github repository](https://github.com/ScaleWeather/eccodes).
//!
//![ecCodes](https://confluence.ecmwf.int/display/ECC/ecCodes+Home) is an open-source library
//!for reading and writing GRIB and BUFR files developed by [European Centre for Medium-Range Weather Forecasts](https://www.ecmwf.int/).
//!
//!## Usage
//!
//!### Accessing GRIB files
//!
//!This crate provides an access to GRIB file by creating a
//![`CodesHandle`] and reading messages from the file with it.
//!
//!The [`CodesHandle`] can be constructed in two ways:
//!
//!- The main option is to use [`new_from_file()`](codes_handle::CodesHandle::new_from_file) function
//!to open a file under provided [`path`](`std::path::Path`) with filesystem,
//!when copying whole file into memory is not desired or not necessary.
//!
//!- Alternatively [`new_from_memory()`](codes_handle::CodesHandle::new_from_memory) function can be used
//!to access a file that is already in memory. For example, when file is downloaded from the internet
//!and does not need to be saved on hard drive.
//!The file must be stored in [`bytes::Bytes`](https://docs.rs/bytes/1.1.0/bytes/struct.Bytes.html).
//!
//!Data (messages) inside the GRIB file can be accessed using the [`FallibleIterator`](`codes_handle::CodesHandle#impl-FallibleIterator`)
//!by iterating over the `CodesHandle`.
//!
//!The `FallibleIterator` returns a [`KeyedMessage` structure which implements some
//!methods to access data values. The data inside `KeyedMessage` is provided directly as [`Key`]
//!or as more specific data type.
//!
//!#### Example
//!
//!```
//!// We are reading the mean sea level pressure for 4 gridpoints
//!// nearest to Reykjavik (64.13N, -21.89E) for 1st June 2021 00:00 UTC
//!// from ERA5 Climate Reanalysis
//!
//!// Open the GRIB file and create the CodesHandle
//!  use eccodes::{ProductKind, CodesHandle, KeyType};
//! # use std::path::Path;
//!  use eccodes::FallibleStreamingIterator;
//! #
//! # fn main() -> anyhow::Result<()> {
//! let file_path = Path::new("./data/iceland.grib");
//! let product_kind = ProductKind::GRIB;
//!
//! let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
//!
//! // Use iterator to get a Keyed message with shortName "msl" and typeOfLevel "surface"
//! // First, filter and collect the messages to get those that we want
//! while let Some(msg) = handle.next()? {
//!     if msg.read_key("shortName")?.value == KeyType::Str("msl".to_string())
//!        && msg.read_key("typeOfLevel")?.value == KeyType::Str("surface".to_string()) {
//!        
//!        // Get the four nearest gridpoints of Reykjavik
//!        let nearest_gridpoints = msg.codes_nearest()?.find_nearest(64.13, -21.89)?;
//!
//!       // Print value and distance of the nearest gridpoint
//!        println!("value: {}, distance: {}",
//!           nearest_gridpoints[3].value,
//!           nearest_gridpoints[3].distance);
//!     }
//! }
//! # Ok(())
//! # }
//!```
//!
//!### Writing GRIB files
//!
//!The crate provides a basic support for setting `KeyedMessage` keys
//!and writing GRIB files. The easiests (and safest) way to create a
//!new custom message is to copy exisitng one from other GRIB file,
//!modify the keys and write to new file.
//!
//!#### Example
//!
//!```rust
//! use eccodes::FallibleStreamingIterator;
//! use eccodes::{CodesHandle, Key, KeyType, ProductKind};
//! # use std::{fs::remove_file, path::Path};
//! 
//! # fn main() -> anyhow::Result<()> {
//!     // We are computing the temperature at 850hPa as an average
//!     // of 900hPa and 800hPa and writing it to a new file.
//!     let file_path = Path::new("./data/iceland-levels.grib");
//!     let mut handle = CodesHandle::new_from_file(file_path, ProductKind::GRIB)?;
//! 
//!     // We need a similar message to edit,
//!     // in this case we can use temperature at 700hPa
//!     let mut new_msg = vec![];
//! 
//!     // Get temperatures at 800hPa and 900hPa
//!     let mut t800 = vec![];
//!     let mut t900 = vec![];
//! 
//!     while let Some(msg) = handle.next()? {
//!         if msg.read_key("shortName")?.value == KeyType::Str("t".to_string()) {
//!             if msg.read_key("level")?.value == KeyType::Int(700) {
//!                 new_msg.push(msg.clone());
//!             }
//! 
//!             if msg.read_key("level")?.value == KeyType::Int(800) {
//!                 if let KeyType::FloatArray(vals) = msg.read_key("values")?.value {
//!                     t800 = vals;
//!                 }
//!             }
//! 
//!             if msg.read_key("level")?.value == KeyType::Int(900) {
//!                 if let KeyType::FloatArray(vals) = msg.read_key("values")?.value {
//!                     t900 = vals;
//!                 }
//!             }
//!         }
//!     }
//! 
//!     let mut new_msg = new_msg.remove(0);
//! 
//!     // Compute temperature at 850hPa
//!     let t850: Vec<f64> = t800
//!         .iter()
//!         .zip(t900.iter())
//!         .map(|t| (t.0 + t.1) / 2.0)
//!         .collect();
//! 
//!     // Edit appropriate keys in the editable message
//!     new_msg.write_key(Key {
//!         name: "level".to_string(),
//!         value: KeyType::Int(850),
//!     })?;
//!     new_msg.write_key(Key {
//!         name: "values".to_string(),
//!         value: KeyType::FloatArray(t850),
//!     })?;
//! 
//!     // Save the message to a new file without appending
//!     new_msg.write_to_file(Path::new("iceland-850.grib"), false)?;
//! 
//! #     remove_file(Path::new("iceland-850.grib")).unwrap();
//! #     Ok(())
//! # }
//!```
//!
//!### Features
//!
//!- `docs` - builds the crate without linking ecCodes, particularly useful when building the documentation
//!on [docs.rs](https://docs.rs/). For more details check documentation of [eccodes-sys](https://crates.io/crates/eccodes-sys).
//!
//!To build your own crate with this crate as dependency on docs.rs without linking ecCodes add following lines to your `Cargo.toml`
//!
//!```text
//![package.metadata.docs.rs]
//!features = ["eccodes/docs"]
//!```
//!

pub mod codes_handle;
#[cfg(feature = "experimental_index")]
#[cfg_attr(docsrs, doc(cfg(feature = "experimental_index")))]
pub mod codes_index;
pub mod codes_nearest;
pub mod errors;
mod intermediate_bindings;
pub mod keyed_message;
pub mod keys_iterator;
#[cfg(feature = "message_ndarray")]
#[cfg_attr(docsrs, doc(cfg(feature = "message_ndarray")))]
pub mod message_ndarray;
mod pointer_guard;

pub use codes_handle::{CodesHandle, ProductKind};
#[cfg(feature = "experimental_index")]
#[cfg_attr(docsrs, doc(cfg(feature = "experimental_index")))]
pub use codes_index::CodesIndex;
pub use codes_nearest::{CodesNearest, NearestGridpoint};
pub use errors::CodesError;
pub use fallible_iterator::{FallibleIterator, IntoFallibleIterator};
pub use fallible_streaming_iterator::FallibleStreamingIterator;
pub use keyed_message::{Key, KeyType, KeyedMessage};
pub use keys_iterator::{KeysIterator, KeysIteratorFlags};
