# eccodes

[![License](https://img.shields.io/github/license/ScaleWeather/eccodes?style=flat-square)](https://choosealicense.com/licenses/apache-2.0/)
[![Crates.io](https://img.shields.io/crates/v/eccodes?style=flat-square)](https://crates.io/crates/eccodes)

**This crate is in very early development stage.** It is currently published to reserve the crate name. First usable release will likely be published in September 2021.

**If you want to contribute or need make this library be released sooner check Github repository.** All submitted issues and pull requests are welcome. 

This crate will contain safe high-level bindings for ecCodes library. Bindings can be considered safe mainly because all crate structures will take ownership of the data in memory before passing the raw pointer to ecCodes.

[ecCodes](https://confluence.ecmwf.int/display/ECC/ecCodes+Home) is an open-source library for reading and writing GRIB and BUFR files developed by [European Centre for Medium-Range Weather Forecasts](https://www.ecmwf.int/).

Because ecCodes supports mainly Linux platforms, this crate is not tested on other architectures.

## Usage

This crate uses [eccodes-sys](https://crates.io/crates/eccodes-sys) with default options to link ecCodes. Check `eccodes-sys` website for more details on how it links the library.

If you would like to build ecCodes with other options simply import `eccodes-sys` along with `eccodes` in your `Cargo.toml` file and select needed features.

This crate will provide to ways of accesing GRIB/BUFR files:
- Using [`fopen()`](https://man7.org/linux/man-pages/man3/fopen.3.html) function to open a file with filesystem, when copying whole file into memory is not desired or not necessary.
- Using ['fmemopen()](https://man7.org/linux/man-pages/man3/fmemopen.3.html) function to use a file that is already in memory. For example, when file is downloaded from the internet and does not need to be saved on hard drive.

## License

The ecCodes library and these bindings are licensed under the [Apache License Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)

