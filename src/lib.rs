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
//!Because ecCodes supports mainly Linux platforms, this crate is not tested on other architectures.
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
//![`CodesHandle`](codes_handle::CodesHandle) and reading messages from the file with it.
//!
//!The [`CodesHandle`](codes_handle::CodesHandle) can be constructed in two ways:
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
//!Data (messages) inside the GRIB file can be accessed using the [`Iterator`](`codes_handle::CodesHandle#impl-Iterator`)
//!by iterating over the `CodesHandle`.
//!
//!The `Iterator` returns a [`KeyedMessage`](codes_handle::KeyedMessage) structure which implements some
//!methods to access data values. The data inside `KeyedMessage` is provided directly as [`Key`](codes_handle::Key)
//!or as more specific data type.
//!
//!### Example
//!
//!```
//!// We are reading the mean sea level pressure in Reykjavik
//!// for 1st June 2021 00:00 UTC (data from ERA5)
//!
//!// Open the GRIB file and create the CodesHandle
//!# use eccodes::codes_handle::{ProductKind, CodesHandle, KeyedMessage};
//!# use eccodes::errors::CodesError;
//!# use std::path::Path;
//!# use eccodes::codes_handle::Key::Str;
//!let file_path = Path::new("./data/iceland.grib");
//!let product_kind = ProductKind::GRIB;
//!
//!let handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();
//!
//!// Use iterator to get a Keyed message with shortName "msl" and typeOfLevel "surface"
//!// First, filter and collect the messages to get those that we want
//!let level: Result<Vec<KeyedMessage>, CodesError> = handle
//!    .filter(|msg| {
//!    let msg = msg.as_ref().unwrap();
//!
//!    msg.read_key("shortName").unwrap() == Str(String::from("msl"))
//!        && msg.read_key("typeOfLevel").unwrap() == Str(String::from("surface"))
//!    })
//!    .collect();
//!
//!// Now unwrap and access the first and only element of resulting vector
//!let level = level.unwrap()[0];
//!
//!// Read the value of KeyedMessage for the grid point nearest of Reykjavik (64N -22E)
//!// Not yet implemented
//!```
//!
//!### ecCodes installation
//!
//!This crate uses [eccodes-sys](https://crates.io/crates/eccodes-sys) with default options to link ecCodes.
//!Check `eccodes-sys` website for more details on how it links the library.
//!
//!The reccomended way to install ecCodes on your computer is using your package manager.
//!For example, on Ubuntu you can use `apt-get`:
//!
//!```bash
//!$ sudo apt-get install libeccodes-dev
//!```
//!
//!Alternatively, you can install the library manually from source in suitable directory
//!following [this instructions](https://confluence.ecmwf.int/display/ECC/ecCodes+installation).
//!
//!Then add the `lib/pkgconfig` directory from your ecCodes installation directory
//!to the `PKG_CONFIG_PATH` environmental variable. For example:
//!
//!```bash
//!$ export PKG_CONFIG_PATH=<your_eccodes_path>/lib/pkgconfig
//!```
//!
//!### Features
//!
//!- `docs` - builds the create without linking ecCodes, particularly useful when building the documentation
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
pub mod errors;
mod intermediate_bindings;
