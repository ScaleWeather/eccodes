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
//!# use eccodes::codes_handle::{ProductKind, CodesHandle, KeyedMessage};
//!# use eccodes::errors::{CodesError};
//!# use std::path::Path;
//!# use eccodes::codes_handle::KeyType::Str;
//!# use eccodes::FallibleIterator;
//!#
//!# fn main() -> Result<(), CodesError> {
//!let file_path = Path::new("./data/iceland.grib");
//!let product_kind = ProductKind::GRIB;
//!
//!let handle = CodesHandle::new_from_file(file_path, product_kind)?;
//!
//!// Use iterator to get a Keyed message with shortName "msl" and typeOfLevel "surface"
//!// First, filter and collect the messages to get those that we want
//!let mut level: Vec<KeyedMessage> = handle
//!    .filter(|msg| {
//!
//!    Ok(msg.read_key("shortName")?.value == Str("msl".to_string())
//!        && msg.read_key("typeOfLevel")?.value == Str("surface".to_string()))
//!    })
//!    .collect()?;
//!
//!// Now unwrap and access the first and only element of resulting vector
//!// Find nearest modifies internal KeyedMessage fields so we need mutable reference
//!let level = &mut level[0];
//!
//!// Get the four nearest gridpoints of Reykjavik
//!let nearest_gridpoints = level.find_nearest(64.13, -21.89)?;
//!
//!// Print value and distance of the nearest gridpoint
//!println!("value: {}, distance: {}",
//!    nearest_gridpoints[3].value,
//!    nearest_gridpoints[3].distance);
//!# Ok(())
//!# }
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
//!# use eccodes::{
//!#     codes_handle::{
//!#         CodesHandle, Key,
//!#         KeyType::{self, FloatArray, Int, Str},
//!#         KeyedMessage,
//!#         ProductKind::{self, GRIB},
//!#     },
//!#     FallibleIterator,
//!# };
//!# use std::{fs::remove_file, path::Path};
//!# use eccodes::errors::CodesError;
//!#
//!# fn main() -> Result<(), CodesError> {
//!// We are computing the temperature at 850hPa as an average
//!// of 900hPa and 800hPa and writing it to a new file.
//!let file_path = Path::new("./data/iceland-levels.grib");
//!let handle = CodesHandle::new_from_file(file_path, GRIB)?;
//!
//!// Get messages with temperature levels
//!let t_levels: Vec<KeyedMessage> = handle
//!    .filter(|msg| Ok(msg.read_key("shortName")?.value == Str("t".to_string())))
//!    .collect()?;
//!
//!// Get any message to edit it later
//!let mut new_msg = t_levels[0].clone();
//!
//!// Get temperatures at 800hPa and 900hPa
//!let mut t800 = vec![];
//!let mut t900 = vec![];
//!
//!for msg in t_levels {
//!    if msg.read_key("level")?.value == Int(800) {
//!        if let FloatArray(vals) = msg.read_key("values")?.value {
//!            t800 = vals;
//!        }
//!    }
//!
//!    if msg.read_key("level")?.value == Int(900) {
//!        if let FloatArray(vals) = msg.read_key("values")?.value {
//!            t900 = vals;
//!        }
//!    }
//!}
//!
//!// Compute temperature at 850hPa
//!let t850: Vec<f64> = t800
//!    .iter()
//!    .zip(t900.iter())
//!    .map(|t| (t.0 + t.1) / 2.0)
//!    .collect();
//!
//!// Edit appropriate keys in the editable message
//!new_msg
//!    .write_key(Key {
//!        name: "level".to_string(),
//!        value: Int(850),
//!    })?;
//!new_msg
//!    .write_key(Key {
//!        name: "values".to_string(),
//!        value: FloatArray(t850),
//!    })?;
//!
//!// Save the message to a new file without appending
//!new_msg
//!    .write_to_file(Path::new("iceland-850.grib"), false)?;
//!#
//!# remove_file(Path::new("iceland-850.grib")).unwrap();
//!# Ok(())
//!# }
//!```
//!
//!### ecCodes installation
//!
//!This crate uses [eccodes-sys](https://crates.io/crates/eccodes-sys) with default options to link ecCodes.
//!Check `eccodes-sys` website for more details on how it links the library.
//!
//!The recommended way to install ecCodes on your computer is using your package manager.
//!For example, on Ubuntu you can use `apt-get`:
//!
//!```text
//!sudo apt-get install libeccodes-dev
//!```
//!
//!or `brew` on MacOS:
//!
//!```text
//!brew install eccodes
//!```
//!
//!Alternatively, you can install the library manually from source in suitable directory
//!following [this instructions](https://confluence.ecmwf.int/display/ECC/ecCodes+installation).
//!
//!Then add the `lib/pkgconfig` directory from your ecCodes installation directory
//!to the `PKG_CONFIG_PATH` environmental variable. If ecCodes have been compiled
//!as shared library you will also need to specify `LD_LIBRARY_PATH`.
//!For example:
//!
//!```text
//!$ export PKG_CONFIG_PATH=<your_eccodes_path>/lib/pkgconfig
//!$ export LD_LIBRARY_PATH=<your_eccodes_path>/lib
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
pub mod keys_iterator;
mod intermediate_bindings;
mod pointer_guard;

pub use codes_handle::{CodesHandle, Key, KeyType, KeyedMessage, ProductKind};
#[cfg(feature = "experimental_index")]
#[cfg_attr(docsrs, doc(cfg(feature = "experimental_index")))]
pub use codes_index::CodesIndex;
pub use codes_nearest::{CodesNearest, NearestGridpoint};
pub use keys_iterator::{KeysIterator, KeysIteratorFlags};
pub use errors::CodesError;
pub use fallible_iterator::{FallibleIterator, IntoFallibleIterator};
pub use fallible_streaming_iterator::FallibleStreamingIterator;
