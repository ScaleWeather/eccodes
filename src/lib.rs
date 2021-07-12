//!# Unofficial high-level safe Rust bindings to ecCodes library.
//!
//!**This crate is in early stage of development.** First usable version will be released soon.
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

mod errors;

use eccodes_sys::{self, _IO_FILE, codes_context, codes_handle};
use libc::{FILE, size_t, c_void, c_char};
use errors::{CodesError, LibcError};
use bytes::Bytes;

///Enum indicating what type of product `CodesHandle` is currently holding.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ProductKind {
    GRIB,
    BUFR,
}

enum DataContainer {
    FileBytes(Bytes),
    FileBuffer(String)
}

///Main structure used to operate on the GRIB/BUFR file.
///It takes a full ownership of the accessed file.
///It can be constructed either using a file (with `fopen()`) or memory buffer (with `fmemopen()`).
pub struct CodesHandle {
    ///The container to take the ownership of handled file
    data: DataContainer,
    ///Internal ecCodes unsafe handle
    handle: *mut codes_handle,
    product_kind: ProductKind,
}

impl CodesHandle {
    pub fn new_from_memory(file_data: Bytes, product_kind: ProductKind) -> Result<Self, CodesError> {
        let file_handle = open_with_fmemopen(&file_data)?;
        let handle = CodesHandle::codes_grib_handle_new_from_file(file_handle)?;

        Ok(CodesHandle {
            data: (DataContainer::FileBytes(file_data)),
            handle,
            product_kind,
        })
    }

    fn codes_grib_handle_new_from_file(mem_handle: *mut FILE) -> Result<*mut codes_handle, CodesError> {
        let context: *mut codes_context = std::ptr::null_mut(); //default context

        let grib_handle;
        let mut error: i32 = 0;
        unsafe {
            grib_handle =
                eccodes_sys::codes_grib_handle_new_from_file(context, mem_handle as *mut _IO_FILE, &mut error as *mut i32);
        }

        if error != 0 {
            return Err(CodesError::Internal);
        }

        Ok(grib_handle)
    }
}

impl Drop for CodesHandle {
    fn drop (&mut self) {
        let error_code;

        unsafe {
            error_code = eccodes_sys::codes_handle_delete(self.handle);
        }
    
        if error_code != 0 {
            panic!("CodesHandle destructor failed with ecCodes error code {:?}!", error_code);
        }
    }
}

fn open_with_fmemopen(grib_file: &Bytes) -> Result<*mut FILE, LibcError> {
    let grib_size = grib_file.len() as size_t;
    let grib_mode = "r".as_ptr().cast::<c_char>();

    let grib_ptr = grib_file.as_ptr() as *mut c_void;

    let grib_obj;
    unsafe {
        grib_obj = libc::fmemopen(grib_ptr, grib_size, grib_mode);
    }

    if grib_obj.is_null() {
        return Err(LibcError::NullPtr);
    }

    Ok(grib_obj)
}
