//!# Unofficial high-level safe Rust bindings to ecCodes library.
//!
//!**Currently only reading of GRIB files is supported.**
//!
//!Check README for more details and how to contribute.
//!
//!## Features
//!
//!- `docs` - builds the create without linking ecCodes, particularly useful when building the documentation
//!on [docs.rs](https://docs.rs/). For more details check documentation of [eccodes-sys](https://crates.io/crates/eccodes-sys).
//!
//!To build your own crate with this crate as dependency on docs.rs without linking ecCodes add following lines to your `Cargo.toml`
//!
//!```Rust
//![package.metadata.docs.rs]
//!features = ["eccodes/docs"]
//!```

mod constructors;
mod errors;
mod destructor;

use bytes::Bytes;
use eccodes_sys::{self, codes_context, codes_handle, _IO_FILE};
use errors::{CodesError, LibcError};
use libc::FILE;

///Enum indicating what type of product `CodesHandle` is currently holding.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ProductKind {
    GRIB,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum DataContainer {
    FileBytes(Bytes),
    FileBuffer(String),
}

///Main structure used to operate on the GRIB/BUFR file.
///It takes a full ownership of the accessed file.
///It can be constructed either using a file (with `fopen()`) or memory buffer (with `fmemopen()`).
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct CodesHandle {
    ///The container to take the ownership of handled file
    data: DataContainer,
    ///Internal ecCodes unsafe handle
    file_handle: *mut codes_handle,
    file_pointer: *mut FILE,
    product_kind: u32,
}

impl CodesHandle {
    fn codes_handle_new_from_file(
        file_pointer: *mut FILE,
        product_kind: u32,
    ) -> Result<*mut codes_handle, CodesError> {
        let context: *mut codes_context = std::ptr::null_mut(); //default context

        let file_handle;
        let mut error_code: i32 = 0;
        unsafe {
            file_handle = eccodes_sys::codes_handle_new_from_file(
                context,
                file_pointer as *mut _IO_FILE,
                product_kind,
                &mut error_code as *mut i32,
            );
        }

        if error_code != 0 {
            return Err(CodesError::Internal(error_code));
        }

        Ok(file_handle)
    }
}
