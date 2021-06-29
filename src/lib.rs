//! Unofficial high-level safe Rust bindings to ecCodes library.
//!
//! **This crate is in early stage of development.** First usable version will be released soon.
//!
//! Check README for more details and how to contribute.

#![allow(dead_code)]
use eccodes_sys::{self, codes_handle};

///Main structure used to operate on the GRIB/BUFR file. 
///It takes a full ownership of the accessed file. 
///It can be constructed either using a file (with `fopen()`) or memory buffer (with `fmemopen()`).
pub struct CodesHandle {
    ///Internal ecCodes unsafe handle
    handle: codes_handle,
}
