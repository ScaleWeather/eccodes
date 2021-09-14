# eccodes

[![License](https://img.shields.io/github/license/ScaleWeather/eccodes)](https://choosealicense.com/licenses/apache-2.0/)
[![Crates.io](https://img.shields.io/crates/v/eccodes)](https://crates.io/crates/eccodes)
[![dependency status](https://deps.rs/repo/github/ScaleWeather/eccodes/status.svg)](https://deps.rs/repo/github/ScaleWeather/eccodes)
[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/ScaleWeather/eccodes/cargo?label=cargo%20build)](https://github.com/ScaleWeather/eccodes/actions)
[![docs.rs](https://img.shields.io/docsrs/eccodes)](https://docs.rs/eccodes)

This crate contains safe high-level bindings for ecCodes library. 
Bindings can be considered safe mainly because all crate structures 
take ownership of the data in memory before passing the raw pointer to ecCodes. 

**Currently only reading of GRIB files is supported.**

As the API of this crate differs significantly from the API of ecCodes library 
make sure to read its [documentation](https://docs.rs/eccodes). 
Read [this section](#crate-safety) to learn more about design decisions of this crate.

**If you want to see more features released quicker do not hesitate to contribute and check out Github repository.** All submitted issues and pull requests are welcome.

[ecCodes](https://confluence.ecmwf.int/display/ECC/ecCodes+Home) is an 
open-source library for reading and writing GRIB and BUFR files 
developed by [European Centre for Medium-Range Weather Forecasts](https://www.ecmwf.int/).

Because ecCodes supports mainly Linux platforms, this crate is not tested on other architectures.

## Usage

### ecCodes installation

This crate uses [eccodes-sys](https://crates.io/crates/eccodes-sys) with default options to link ecCodes.
Check `eccodes-sys` website for more details on how it links the library.

The reccomended way to install ecCodes on your computer is using your package manager.
For example, on Ubuntu you can use `apt-get`:

```bash
$ sudo apt-get install libeccodes-dev
```

Alternatively, you can install the library manually from source in suitable directory
following [this instructions](https://confluence.ecmwf.int/display/ECC/ecCodes+installation).

Then add the `lib/pkgconfig` directory from your ecCodes installation directory
to the `PKG_CONFIG_PATH` environmental variable. For example:

```bash
$ export PKG_CONFIG_PATH=<your_eccodes_path>/lib/pkgconfig
```

### Accessing GRIB files

This crate provides an access to GRIB file by creating a
`CodesHandle` and reading messages from the file with it.

The `CodesHandle` can be constructed in two ways:

- The main option is to use `new_from_file()` function
to open a file under provided `path` with filesystem,
when copying whole file into memory is not desired or not necessary.

- Alternatively `new_from_memory()` function can be used
to access a file that is already in memory. For example, when file is downloaded from the internet
and does not need to be saved on hard drive. 
The file must be stored in `bytes::Bytes`.

Data (messages) inside the GRIB file can be accessed using the `Iterator`
by iterating over the `CodesHandle`.

The `Iterator` returns a `KeyedMessage` structure which implements some
methods to access data values. The data inside `KeyedMessage` is provided directly as `Key`
or as more specific data type.

### Example

```rust
// We are reading the mean sea level pressure in Reykjavik
// for 1st June 2021 00:00 UTC (data from ERA5)

// Open the GRIB file and create the CodesHandle
let file_path = Path::new("./data/iceland.grib");
let product_kind = ProductKind::GRIB;

let handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();

// Use iterator to get a Keyed message with shortName "msl" and typeOfLevel "surface"
// First, filter and collect the messages to get those that we want
let level: Result<Vec<KeyedMessage>, CodesError> = handle
    .filter(|msg| {
    let msg = msg.as_ref().unwrap();

    msg.read_key("shortName").unwrap() == Str(String::from("msl"))
        && msg.read_key("typeOfLevel").unwrap() == Str(String::from("surface"))
    })
    .collect();

// Now unwrap and access the first and only element of resulting vector
let level = level.unwrap()[0];

// Read the value of KeyedMessage for the grid point nearest of Reykjavik (64N -22E)
// Not yet implemented
```

### Features

- `docs` - builds the create without linking ecCodes, particularly useful when building the documentation
on [docs.rs](https://docs.rs/). For more details check documentation of [eccodes-sys](https://crates.io/crates/eccodes-sys).

To build your own crate with this crate as dependency on docs.rs without linking ecCodes add following lines to your `Cargo.toml`

```toml
[package.metadata.docs.rs]
features = ["eccodes/docs"]
```

## Crate safety

Because the ecCodes library API heavily relies on raw pointers simply making ecCodes functions callable without `unsafe` block would still allow for creation of dangling pointers and use-after-free, and the crate would not be truly safe. Therefore these bindings are rather thick wrapper as they need to take full ownership of accessed data to make the code safe. Having the data and pointers contained in dedicated data structures is also an occassion to make this crate API more convienient to use than the original ecCodes API (which is not really user-friendly).

## Roadmap

_(Functions from ecCodes API wrapped at given stage are marked in parentheses)_

- [ ] Reading GRIB files
    - [x] Creating CodesHandle from file and from memory (`codes_handle_new_from_file`, `codes_handle_delete`)
    - [x] Iterating over GRIB messages with `Iterator`
    - [x] Reading keys from messages (`codes_get_double`, `codes_get_long`, `codes_get_string`, `codes_get_double_array`, `codes_get_long_array`, `codes_get_size`, `codes_get_length`, `codes_get_native_type`)
    - [ ] Iterating over key names with `Iterator` (`codes_grib_iterator_new`, `codes_grib_iterator_next`, `codes_keys_iterator_get_name`, `codes_keys_iterator_rewind `, `codes_grib_iterator_delete`)
    - [ ] Iterating over latitude/longitude/values with `Iterator` (`codes_grib_iterator_new`, `codes_grib_get_data`, `codes_grib_iterator_next`, `codes_grib_iterator_previous`, `codes_grib_iterator_has_next`, `codes_grib_iterator_reset`, `codes_grib_iterator_delete`)
    - [ ] Finding nearest data points for given coordinates (`codes_grib_nearest_new`, `codes_grib_nearest_find`, `codes_grib_nearest_delete`, `codes_grib_nearest_find_multiple`)
- [ ] Writing GRIB files
- [ ] Reading and writing BUFR files

## License

The ecCodes library and these bindings are licensed under the [Apache License Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)

