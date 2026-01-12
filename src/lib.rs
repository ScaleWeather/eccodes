#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_wrap)]
#![warn(missing_docs)]
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
//! **Version 0.14 introduces breaking changes, [check them below](#changes-in-version-014)!**
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
//! To access a GRIB file you need to create [`CodesFile`] with one of the provided constructors.
//!
//! ecCodes represents GRIB files as a set of separate messages, each containing data fields at specific time and level.
//! Messages are represented here by a generic [`CodesMessage`](codes_message::CodesMessage) structure, but you shouldn't use it directly.
//! Instead use [`RefMessage`], [`ArcMessage`] or [`BufMessage`] to operations - check the docs for more information when to use each.
//!
//! To obtain `CodesMessage` from `CodesFile` you need to create an instance of [`RefMessageIter`] or [`ArcMessageIter`] using
//! [`CodesFile::ref_message_iter()`] or [`CodesFile::arc_message_iter()`].
//! Those structures implement [`FallibleIterator`], please check its documentation if you are not familiar with it.
//!
//! `CodesMessage` implements several methods to access the data as needed, most of those can be called directly.
//! Almost all methods can be called on any `CodesMessage`, except for [`KeyWrite`] operations, which can be called only on [`BufMessage`]
//! to avoid confusion if written keys are save to file or not.
//!
//! Data contained by `KeyedMessage` is represented as *keys* (like in dictionary).
//! Keys can be read with static types using [`read_key()`](KeyRead::read_key) or with [dynamic types](codes_message::DynamicKeyType)
//! using [`read_key_dynamic()`](codes_message::CodesMessage::read_key_dynamic).
//! To discover what keys are present in a message use [`KeysIterator`](KeysIterator).
//!
//! With `ndarray` feature (enabled by default) you can also read `CodesMessage` into `ndarray` using [`to_ndarray()`](codes_message::CodesMessage::to_ndarray)
//! and [`to_lons_lats_values()`](codes_message::CodesMessage::to_lons_lats_values).
//!
//! You can use [`CodesNearest`] to get the data values of four nearest gridpoints for given coordinates.
//!
//! To modify keys within the message use [`write_key_unchecked()`](KeyWrite::write_key_unchecked).
//! To save that modified message use [`write_to_file()`](codes_message::CodesMessage::write_to_file).
//!
//! #### Example 1 - Reading GRIB file
//!
//! In this example we are reading mean sea level pressure for 4 gridpoints nearest to
//! Reykjavik (64.13N, -21.89E) for 1st June 2021 00:00 UTC from ERA5 Climate Reanalysis.
//!
//! ```
//! use eccodes::{CodesFile, FallibleIterator, KeyRead, ProductKind};
//!
//! # fn main() -> anyhow::Result<()> {
//! // Open the GRIB file and create the CodesHandle
//! let mut handle = CodesFile::new_from_file("./data/iceland.grib", ProductKind::GRIB)?;
//! // Use iterator to find a message with shortName "msl" and typeOfLevel "surface"
//! // We can use while let or for_each() to iterate over the messages
//! while let Some(msg) = handle.ref_message_iter().next()? {
//!     // We need to specify the type of key we read
//!     let short_name: String = msg.read_key("shortName")?;
//!     let type_of_level: String = msg.read_key("typeOfLevel")?;
//!     if short_name == "msl" && type_of_level == "surface" {
//!         // Create CodesNearest for given message
//!         let nearest_gridpoints = msg
//!             .codes_nearest()?
//!             // Find the nearest gridpoints to Reykjavik
//!             .find_nearest(64.13, -21.89)?;
//!         // Print value and distance of the nearest gridpoint
//!         println!(
//!             "value: {}, distance: {}",
//!             nearest_gridpoints[3].value, nearest_gridpoints[3].distance
//!         );
//!     }
//! }
//! #   Ok(())
//! # }
//! ```
//!
//! #### (New in 0.14) Example 2 - Concurrent read
//!
//! This example shows how `ArcMessage` can be used to do concurrent operations
//! on different message within one file and on the same message as well.
//!
//! ```
//! use eccodes::{CodesError, CodesFile, FallibleIterator, KeyRead, ProductKind};
//! use std::sync::Arc;
//!
//! # fn main() -> anyhow::Result<()> {
//! // We open the file as before
//! let handle = CodesFile::new_from_file("./data/iceland-levels.grib", ProductKind::GRIB)?;
//!
//! // Note different mutability - RefMessageIter required the handle to be mutable
//! let mut arc_msg_gen = handle.arc_message_iter();
//!
//! let mut join_handles = vec![];
//!
//! while let Some(msg) = arc_msg_gen.next()? {
//!     // ArcMessage is Send+Sync
//!     let msg1 = Arc::new(msg);
//!     let msg2 = msg1.clone();
//!
//!     // For each message we spawn two threads then we do operations simulataneously
//!     join_handles.push(std::thread::spawn(move || msg1.read_key("shortName")));
//!     join_handles.push(std::thread::spawn(move || msg2.read_key("typeOfLevel")));
//! }
//!
//! // Now we collect and print the results
//! let read_results = join_handles
//!     .into_iter()
//!     // In production you should probably handle this join error
//!     .map(|jh| jh.join().unwrap())
//!     .collect::<Result<Vec<String>, CodesError>>()?;
//!
//! println!("{read_results:?}");
//!
//! # Ok(())
//! # }
//! ```
//!
//! #### Example 3 - Writing GRIB files and indexing
//!
//! The crate provides basic support for setting `KeyedMessage` keys
//! and writing GRIB files. To create a new custom message we need to copy
//! existing one from other GRIB file, modify the keys and write to new file.
//!
//! In this example we are computing the temperature at 850hPa as an average
//! of 900hPa and 800hPa and writing it to a new file. This example also shows
//! how you can create a simple index for a file to get messages we need.
//! To read values from a message you can also use [`to_ndarray()`](codes_message::CodesMessage::to_ndarray).
//!
//! ```
//! use anyhow::Context;
//! use eccodes::{CodesFile, FallibleIterator, KeyRead, KeyWrite, ProductKind};
//! use std::{collections::HashMap, fs::remove_file, path::Path};
//! 
//! # fn main() -> anyhow::Result<()> {
//! // Start by opening the file and creating CodesHandle
//! let file_path = Path::new("./data/iceland-levels.grib");
//! let mut handle = CodesFile::new_from_file(file_path, ProductKind::GRIB)?;
//! 
//! // To build the index we need to collect all messages
//! let messages = handle.ref_message_iter().collect::<Vec<_>>()?;
//! let mut msg_index = HashMap::new();
//! msg_index.reserve(messages.len());
//! 
//! // Now we can put the messages into a hashmap and index them by shortName and level
//! for msg in messages.into_iter() {
//!     // all messages in this grib are on the same level type
//!     let short_name: String = msg.read_key("shortName")?;
//!     let level: i64 = msg.read_key("level")?;
//! 
//!     msg_index.insert((short_name, level), msg);
//! }
//! 
//! // Now we can get the values from messages we need
//! let t_800: Vec<f64> = msg_index
//!     .get(&("t".to_string(), 800))
//!     .context("message missing")? // we use anyhow context for convienience
//!     .read_key("values")?;
//! let t_900: Vec<f64> = msg_index
//!     .get(&("t".to_string(), 800))
//!     .context("message missing")?
//!     .read_key("values")?;
//! 
//! // We will also clone t at 700hPa to edit it
//! let mut t_850_msg = msg_index
//!     .get(&("t".to_string(), 700))
//!     .context("message missing")?
//!     .try_clone()?;
//! 
//! // Compute temperature at 850hPa
//! let t_850_values: Vec<f64> = t_800
//!     .iter()
//!     .zip(t_900.iter())
//!     .map(|t| (t.0 + t.1) / 2.0)
//!     .collect();
//! 
//! // Edit appropriate keys in the cloned (editable) message
//! t_850_msg.write_key_unchecked("level", 850)?;
//! t_850_msg.write_key_unchecked("values", t_850_values.as_slice())?;
//! 
//! // Save the message to a new file without appending
//! t_850_msg.write_to_file(Path::new("iceland-850.grib"), false)?;
//!    #  remove_file(Path::new("iceland-850.grib")).unwrap();
//!    #  Ok(())
//! # }
//! ```
//!
//! ## Changes in version 0.14
//! 
//! 1. `experimental_index` feature has been removed - users are encouraged to create their own indexes as shown above or use iterator filtering
//! 2. `message_ndarray` feature has been renamed to `ndarray`
//! 3. `CodesHandle` has been renamed to `CodesFile`
//! 4. `KeyedMessage` has been replaced with generic `CodesMessage` - `RefMessage` has the most similar behaviour to `KeyedMessage`]
//! 5. `write_key` is now `write_key_unchecked`
//! 6. Dependency on `FallibleStreamingIterator` has been removed
//!
//! ## Feature Flags
//!
//! - `ndarray` - enables support for converting [`CodesMessage`](codes_message::CodesMessage) to [`ndarray::Array`].
//!   This feature is enabled by default. It is currently tested only with simple lat-lon grids.
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

pub mod codes_file;
pub mod codes_message;
pub mod codes_nearest;
pub mod errors;
mod intermediate_bindings;
pub mod keys_iterator;

mod pointer_guard;

pub use codes_file::{ArcMessageIter, CodesFile, ProductKind, RefMessageIter};
pub use codes_message::{ArcMessage, BufMessage, KeyRead, KeyWrite, RefMessage};
pub use codes_nearest::{CodesNearest, NearestGridpoint};
pub use errors::CodesError;
pub use fallible_iterator::{FallibleIterator, IntoFallibleIterator};
pub use keys_iterator::{KeysIterator, KeysIteratorFlags};
