#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_wrap)]
#![warn(missing_docs)]
#![warn(clippy::cargo)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # Unofficial high-level safe Rust bindings to ecCodes library
//! 
//! [![Github Repository](https://img.shields.io/badge/Github-Repository-blue?style=flat-square&logo=github&color=blue)](https://github.com/ScaleWeather/eccodes)
//! [![Crates.io](https://img.shields.io/crates/v/eccodes?style=flat-square)](https://crates.io/crates/eccodes)
//! [![License](https://img.shields.io/github/license/ScaleWeather/eccodes?style=flat-square)](https://choosealicense.com/licenses/apache-2.0/) \
//! [![dependency status](https://deps.rs/repo/github/ScaleWeather/eccodes/status.svg?style=flat-square)](https://deps.rs/repo/github/ScaleWeather/eccodes)
//! ![Crates.io MSRV](https://img.shields.io/crates/msrv/eccodes?style=flat-square)
//! ![ecCodes version](https://img.shields.io/badge/ecCodes-%E2%89%A52.24.0-blue?style=flat-square&color=blue)
//! 
//! This crate contains (mostly) safe high-level bindings for ecCodes library.
//! Bindings can be considered safe mainly because all crate structures
//! will take ownership of the data in memory before passing the raw pointer to ecCodes.
//! 
//! **Currently only reading of GRIB files is supported.**
//! 
//! Because of the ecCodes library API characteristics theses bindings are
//! rather thick wrapper to make this crate safe and convenient to use.
//! 
//! This crate officially supports mainly Linux platforms same as the ecCodes library.
//! But it is possible to install ecCodes on `MacOS` and this crate successfully compiles and all tests pass.
//! 
//! If you want to see more features released quicker do not hesitate
//! to contribute and check out [Github repository](https://github.com/ScaleWeather/eccodes).
//! 
//! [ecCodes](https://confluence.ecmwf.int/display/ECC/ecCodes+Home) is an open-source library
//! for reading and writing GRIB and BUFR files developed by [European Centre for Medium-Range Weather Forecasts](https://www.ecmwf.int/).
//! 
//! ## Errors and panics
//! 
//! This crate aims to return error whenever possible, even if the error is caused by implementation bug.
//! As ecCodes is often used in scientific applications with long and extensive jobs,
//! this allows the user to handle the error in the way that suits them best and not risk crashes.
//! 
//! All error descriptions are provided in the [`errors`] module.
//! Destructors, which cannot panic, report errors through the `log` crate.
//! 
//! None of the functions in this crate explicitly panics.
//! However, users should not that dependencies might panic in some edge cases.
//! 
//! ## Safety
//! 
//! This crate aims to be as safe as possible and a lot of effort has been put into testing its safety.
//! Moreover, pointers are always checked for null before being dereferenced.
//! 
//! That said, neither main developer nor contributors have expertise in unsafe Rust and bugs might have
//! slipped through. We are also not responsible for bugs in the ecCodes library.
//! 
//! If you find a bug or have a suggestion, feel free to discuss it on Github.
//! 
//! ## Features
//! 
//! - `message_ndarray` - enables support for converting [`KeyedMessage`] to [`ndarray::Array`].
//! This feature is enabled by default. It is currently tested only with simple lat-lon grids.
//! 
//! - `experimental_index` - enables support for creating and using index files for GRIB files.
//! This feature experimental and disabled by default. If you want to use it, please read
//! the information provided in [`codes_index`] documentation.
//! 
//! - `docs` - builds the crate without linking ecCodes, particularly useful when building the documentation
//! on [docs.rs](https://docs.rs/). For more details check documentation of [eccodes-sys](https://crates.io/crates/eccodes-sys).
//! 
//! To build your own crate with this crate as dependency on docs.rs without linking ecCodes add following lines to your `Cargo.toml`
//! 
//! ```text
//! [package.metadata.docs.rs]
//! features = ["eccodes/docs"]
//! ```
//! 
//! ## Usage
//! 
//! To access a GRIB file you need to create [`CodesHandle`] with one of provided constructors.
//! 
//! GRIB files consist of messages which represent data fields at specific time and level.
//! Messages are represented by the [`KeyedMessage`] structure.
//! 
//! [`CodesHandle`] implements [`FallibleStreamingIterator`](CodesHandle#impl-FallibleStreamingIterator-for-CodesHandle<GribFile>)
//! which allows you to iterate over messages in the file. The iterator returns `&KeyedMessage` which valid until next iteration.
//! `KeyedMessage` implements several methods to access the data as needed, most of those can be called directly on `&KeyedMessage`.
//! You can also use [`try_clone()`](KeyedMessage::try_clone) to clone the message and prolong its lifetime.
//! 
//! Data defining and contained by `KeyedMessage` is represented by [`Key`]s. 
//! You can read them directly with [`read_key()`](KeyedMessage::read_key), use [`KeysIterator`](KeyedMessage) 
//! to iterate over them or use [`CodesNearest`] to get the values of four nearest gridpoints for given coordinates.
//! 
//! You can also modify the message with [`write_key()`](KeyedMessage::write_key) and write 
//! it to a new file with [`write_to_file()`](KeyedMessage::write_to_file).
//! 
//! #### Example 1 - Reading GRIB file
//! 
//! ```
//! // We are reading the mean sea level pressure for 4 gridpoints
//! // nearest to Reykjavik (64.13N, -21.89E) for 1st June 2021 00:00 UTC
//! // from ERA5 Climate Reanalysis
//! 
//! use eccodes::{ProductKind, CodesHandle, KeyType};
//! # use std::path::Path;
//! use eccodes::FallibleStreamingIterator;
//! #
//! # fn main() -> anyhow::Result<()> {
//! 
//! // Open the GRIB file and create the CodesHandle
//! let file_path = Path::new("./data/iceland.grib");
//! let product_kind = ProductKind::GRIB;
//! let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
//! 
//! // Use iterator to find a message with shortName "msl" and typeOfLevel "surface"
//! // We can use while let or for_each() to iterate over the messages
//! while let Some(msg) = handle.next()? {
//!     if msg.read_key("shortName")?.value == KeyType::Str("msl".to_string())
//!         && msg.read_key("typeOfLevel")?.value == KeyType::Str("surface".to_string()) {
//!        
//!         // Create CodesNearest for given message
//!         let nearest_gridpoints = msg.codes_nearest()?
//!             // Find the nearest gridpoints to Reykjavik
//!             .find_nearest(64.13, -21.89)?;
//!
//!         // Print value and distance of the nearest gridpoint
//!         println!("value: {}, distance: {}",
//!             nearest_gridpoints[3].value,
//!             nearest_gridpoints[3].distance);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//! 
//! #### Example 2 - Writing GRIB files
//! 
//! ```rust
//! // The crate provides basic support for setting `KeyedMessage` keys
//! // and writing GRIB files. The easiests (and safest) way to create a
//! // new custom message is to copy exisitng one from other GRIB file,
//! // modify the keys and write to new file.
//! 
//! // Here we are computing the temperature at 850hPa as an average
//! // of 900hPa and 800hPa and writing it to a new file.
//! 
//! use eccodes::FallibleStreamingIterator;
//! use eccodes::{CodesHandle, Key, KeyType, ProductKind};
//! # use std::{fs::remove_file, path::Path};
//!  
//! # fn main() -> anyhow::Result<()> {
//! // Start by opening the file and creating CodesHandle
//! let file_path = Path::new("./data/iceland-levels.grib");
//! let mut handle = CodesHandle::new_from_file(file_path, ProductKind::GRIB)?;
//!
//! // We need a message to edit,in this case we can use 
//! // temperature at 700hPa, which is similar to our result
//! let mut new_msg = vec![];
//!
//! // Get data values of temperatures at 800hPa and 900hPa
//! let mut t800 = vec![];
//! let mut t900 = vec![];
//!
//! // Iterate over the messages and collect the data to defined vectors
//! while let Some(msg) = handle.next()? {
//!     if msg.read_key("shortName")?.value == KeyType::Str("t".to_string()) {
//!         if msg.read_key("level")?.value == KeyType::Int(700) {
//!            // To use message outside of the iterator we need to clone it
//!             new_msg.push(msg.try_clone()?);
//!         }
//!
//!         if msg.read_key("level")?.value == KeyType::Int(800) {
//!             if let KeyType::FloatArray(vals) = msg.read_key("values")?.value {
//!                 t800 = vals;
//!             }
//!         }
//!
//!         if msg.read_key("level")?.value == KeyType::Int(900) {
//!             if let KeyType::FloatArray(vals) = msg.read_key("values")?.value {
//!                 t900 = vals;
//!             }
//!         }
//!     }
//! }
//!
//! // This converts the vector to a single message
//! let mut new_msg = new_msg.remove(0);
//!
//! // Compute temperature at 850hPa
//! let t850: Vec<f64> = t800
//!     .iter()
//!     .zip(t900.iter())
//!     .map(|t| (t.0 + t.1) / 2.0)
//!     .collect();
//!
//! // Edit appropriate keys in the editable message
//! new_msg.write_key(Key {
//!     name: "level".to_string(),
//!     value: KeyType::Int(850),
//! })?;
//! new_msg.write_key(Key {
//!     name: "values".to_string(),
//!     value: KeyType::FloatArray(t850),
//! })?;
//!
//! // Save the message to a new file without appending
//! new_msg.write_to_file(Path::new("iceland-850.grib"), false)?;
//!
//! # remove_file(Path::new("iceland-850.grib")).unwrap();
//! # Ok(())
//! # }
//! ```
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
pub use keyed_message::{DynamicKey, DynamicKeyType, KeyedMessage, KeyRead, KeyWrite};
pub use keys_iterator::{KeysIterator, KeysIteratorFlags};
