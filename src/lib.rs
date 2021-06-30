//!# Unofficial high-level safe Rust bindings to ecCodes library.
//!
//!**This crate is in early stage of development.** First usable version will be released soon.
//!
//!Check README for more details and how to contribute.
//!
//!## Features
//!
//!- 'docs` - builds the create without linking ecCodes, particularly useful when building the documentation
//!on [docs.rs](https://docs.rs/). For more details check documentation of [eccodes-sys](https://crates.io/crates/eccodes-sys).
//!
//!To build your own crate with this crate as dependency on docs.rs without linking ecCodes add following lines to your `Cargo.toml`
//!
//!```Rust
//![package.metadata.docs.rs]
//!features = ["eccodes/docs"]
//!```

#![allow(dead_code)]
use eccodes_sys::{self, codes_handle};

///Main structure used to operate on the GRIB/BUFR file. 
///It takes a full ownership of the accessed file. 
///It can be constructed either using a file (with `fopen()`) or memory buffer (with `fmemopen()`).
pub struct CodesHandle {
    ///Internal ecCodes unsafe handle
    handle: codes_handle,
}
