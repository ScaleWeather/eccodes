#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_wrap)]
// #![warn(missing_docs)]
#![warn(clippy::cargo)]
#![warn(clippy::perf)]
#![warn(clippy::doc_lazy_continuation)]
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
//! take ownership of the data in memory before passing the raw pointer to the ecCodes.
//!
//! **Currently only operations on GRIB files are supported.**
//!
//! **Version 0.14 introduces some breaking changes, [check them below](#changes-in-version-014)!**
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
//! Destructors, which cannot panic, report errors through `tracing` and `log` crate.
//!
//! None of the functions in this crate explicitly panics.
//! However, users should be aware that dependencies (eg. `ndarray`) might panic in some edge cases.
//!
//! ## Safety
//!
//! This crate aims to be as safe as possible and a lot of effort has been put into testing its safety.
//! Moreover, pointers are always checked for null before being dereferenced.
//!
//! That said, neither main developer nor contributors have expertise in unsafe Rust and bugs might have
//! slipped through. We are also not responsible for bugs in the ecCodes library.
//!
//! **For critical applications always perform extensive testing before using this crate in production.**
//!
//! If you find a bug or have a suggestion, feel free to discuss it on Github.
//!
//! ## Usage
//!
//! To access a GRIB file you need to create [`CodesHandle`] with one of provided constructors.
//!
//! ecCodes represents GRIB files as a set of separate messages, each representing data fields at specific time and level.
//! Messages are represented here by the [`KeyedMessage`] structure.
//!
//! To obtain `KeyedMessage`(s) from `CodesHandle` you need to create an instance of [`KeyedMessageGenerator`](codes_handle::KeyedMessageGenerator)
//! with [`CodesHandle::message_generator()`]. This is an analogous interface to `IterMut` and `iter_mut()` in [`std::slice`].
//!
//! `KeyedMessageGenerator` implements [`FallibleIterator`](codes_handle::KeyedMessageGenerator#impl-FallibleIterator-for-KeyedMessageGenerator%3C'ch,+S%3E)
//! which allows you to iterate over messages in the file. The iterator returns `KeyedMessage` with lifetime tied to the lifetime of `CodesHandle`,
//! that is `KeyedMessage` cannot outlive the `CodesHandle` it was generated from. If you need to prolong its lifetime, you can use
//! [`try_clone()`](KeyedMessage::try_clone), but that comes with performance and memory overhead.
//!
//! `KeyedMessage` implements several methods to access the data as needed, most of those can be called directly on `&KeyedMessage`.
//! You can also use [`try_clone()`](KeyedMessage::try_clone) to clone the message and prolong its lifetime.
//!
//! Data contained by `KeyedMessage` is represented as *keys* (like in dictionary).
//! Keys can be read with static types using [`read_key()`](KeyedMessage::read_key) or with [dynamic types](`DynamicKeyType`)
//! using [`read_key_dynamic()`](KeyedMessage::read_key_dynamic). To discover what keys are present in a message use [`KeysIterator`](KeyedMessage).
//!
//! You can use [`CodesNearest`] to get the data values of four nearest gridpoints for given coordinates.
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
//! use eccodes::{ProductKind, CodesHandle, KeyRead};
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
//!     // We need to specify the type of key we read
//!     let short_name: String = msg.read_key("shortName")?;
//!     let type_of_level: String = msg.read_key("typeOfLevel")?;
//!
//!     if short_name == "msl" && type_of_level == "surface" {
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
//! #### **New in 0.14** Example 3: Concurrent read
//!
//!
//! #### Example 3 - Writing GRIB files
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
//! use eccodes::{CodesHandle, KeyRead, KeyWrite, ProductKind};
//! # use std::{fs::remove_file, path::Path};
//!  
//! # fn main() -> anyhow::Result<()> {
//! // Start by opening the file and creating CodesHandle
//! let file_path = Path::new("./data/iceland-levels.grib");
//! let mut handle = CodesHandle::new_from_file(file_path, ProductKind::GRIB)?;
//!
//! // We need a message to edit, in this case we can use
//! // temperature at 700hPa, which is similar to our result
//! let mut new_msg = vec![];
//!
//! // Get data values of temperatures at 800hPa and 900hPa
//! let mut t800: Vec<f64> = vec![];
//! let mut t900: Vec<f64> = vec![];
//!
//! // Iterate over the messages and collect the data to defined vectors
//! while let Some(msg) = handle.next()? {
//!     let short_name: String = msg.read_key("shortName")?;
//!
//!     if short_name == "t" {
//!         let level: i64 = msg.read_key("level")?;
//!
//!         if level == 700 {
//!            // To use message outside of the iterator we need to clone it
//!             new_msg.push(msg.try_clone()?);
//!         }
//!
//!         if level == 800 {
//!             t800 = msg.read_key("values")?;
//!         }
//!
//!         if level == 900 {
//!             t900 = msg.read_key("values")?;
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
//! new_msg.write_key("level", 850)?;
//! new_msg.write_key("values", &t850)?;
//!
//! // Save the message to a new file without appending
//! new_msg.write_to_file(Path::new("iceland-850.grib"), false)?;
//!
//! # remove_file(Path::new("iceland-850.grib")).unwrap();
//! # Ok(())
//! # }
//! ```
//!

//!
//!
//! ## Changes in version 0.14
//!
//!
//! ## Feature Flags
//!
//! - `ndarray` - enables support for converting [`KeyedMessage`] to [`ndarray::Array`].
//!   This feature is enabled by default. It is currently tested only with simple lat-lon grids.
//!
//! - `experimental_index` - **⚠️ This feature is experimental and might be unsafe in some contexts ⚠️**
//!   This flag enables support for creating and using index files for GRIB files.
//!   If you want to use it, please read the information provided in [`codes_index`] documentation.
//!
//! - `docs` - builds the crate without linking ecCodes, particularly useful when building the documentation
//!   on [docs.rs](https://docs.rs/). For more details check documentation of [eccodes-sys](https://crates.io/crates/eccodes-sys).
//!
//! To build your own crate with this crate as dependency on docs.rs without linking ecCodes add following lines to your `Cargo.toml`
//!
//! ```text
//! [package.metadata.docs.rs]
//! features = ["eccodes/docs"]
//! ```

pub mod codes_handle;
pub mod codes_message;
pub mod codes_nearest;
pub mod errors;
mod intermediate_bindings;
pub mod keys_iterator;
#[cfg(feature = "ndarray")]
#[cfg_attr(docsrs, doc(cfg(feature = "ndarray")))]
mod message_ndarray;
mod pointer_guard;

pub use codes_handle::{CodesFile, ProductKind};
pub use codes_message::{ArcMessage, BufMessage, KeyRead, KeyWrite, RefMessage};
pub use codes_nearest::{CodesNearest, NearestGridpoint};
pub use errors::CodesError;
pub use fallible_iterator::{FallibleIterator, IntoFallibleIterator};
pub use keys_iterator::{KeysIterator, KeysIteratorFlags};
