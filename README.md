# eccodes

[![License](https://img.shields.io/github/license/ScaleWeather/eccodes)](https://choosealicense.com/licenses/apache-2.0/)
[![Crates.io](https://img.shields.io/crates/v/eccodes)](https://crates.io/crates/eccodes)
[![dependency status](https://deps.rs/crate/eccodes/0.0.2/status.svg)](https://deps.rs/crate/eccodes)

This crate contains safe high-level bindings for ecCodes library. Bindings can be considered safe mainly because all crate structures will take ownership of the data in memory before passing the raw pointer to ecCodes. **Currently only reading of GRIB files is supported.**

**If you want to see more features released quicker do not hesitate to contribute and check out Github repository.** All submitted issues and pull requests are welcome.

Because of the ecCodes library API characteristics theses bindings are rather thick wrapper to make this crate safe and convienient to use.

[ecCodes](https://confluence.ecmwf.int/display/ECC/ecCodes+Home) is an open-source library for reading and writing GRIB and BUFR files developed by [European Centre for Medium-Range Weather Forecasts](https://www.ecmwf.int/).

Because ecCodes supports mainly Linux platforms, this crate is not tested on other architectures.

## Usage

This crate uses [eccodes-sys](https://crates.io/crates/eccodes-sys) with default options to link ecCodes.
Check `eccodes-sys` website for more details on how it links the library.

If you would like to build ecCodes with other options simply import `eccodes-sys`
along with `eccodes` in your `Cargo.toml` file and select needed features.

For example:

```toml
[dependencies]
eccodes = "0.1.0"
eccodes-sys = { version="0.1.3", features=["build_source"] }
```

### Accessing GRIB files

This crate provides an access to GRIB file by creating a
`CodesHandle` codes_handle::CodesHandle and reading messages from the file with it.

The `CodesHandle` codes_handle::CodesHandle can be constructed in two ways:

- The main option is to use `new_from_file()` function
to open a file under provided `path` with filesystem,
when copying whole file into memory is not desired or not necessary.

- Alternatively `new_from_memory()` function can be used
to access a file that is already in memory. For example, when file is downloaded from the internet
and does not need to be saved on hard drive. 
The file must be stored in `bytes::Bytes`.

Then we can iterate with `Iterator` over the `CodesHandle` to read `KeyedMessage`.

### Example

```rust
// We are reading the mean sea level pressure in Reykjavik
// for 1st June 2021 00:00 UTC (data from ERA5)

// Open the GRIB file and create the CodesHandle
let file_path = PathBuf::from("./data/iceland.grib");
let product_kind = ProductKind::GRIB;

let handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();

// Iterate to the KeyedMessage with shortName "msl" and typeOfLevel "surface"

// Read the value of KeyedMessage for the grid point nearest of Reykjavik
// 64N -22E
```

## Features

- `docs` - builds the create without linking ecCodes, particularly useful when building the documentation
on [docs.rs](https://docs.rs/). For more details check documentation of [eccodes-sys](https://crates.io/crates/eccodes-sys).

To build your own crate with this crate as dependency on docs.rs without linking ecCodes add following lines to your `Cargo.toml`

```toml
[package.metadata.docs.rs]
features = ["eccodes/docs"]
```

## License

The ecCodes library and these bindings are licensed under the [Apache License Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)

