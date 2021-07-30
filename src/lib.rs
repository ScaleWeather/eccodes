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

mod errors;

use std::ffi::CString;

use bytes::Bytes;
use eccodes_sys::{
    self, codes_context, codes_handle, ProductKind_PRODUCT_GRIB, _IO_FILE,
};
use errno::errno;
use errors::{CodesError, LibcError};
use libc::{c_char, c_void, size_t, FILE};
use log::error;

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
    pub fn new_from_memory(
        file_data: Bytes,
        product_kind: ProductKind,
    ) -> Result<Self, CodesError> {
        let product_kind = match_product_kind(product_kind);

        let file_pointer = open_with_fmemopen(&file_data)?;
        let file_handle = CodesHandle::codes_handle_new_from_file(file_pointer, product_kind)?;

        Ok(CodesHandle {
            data: (DataContainer::FileBytes(file_data)),
            file_handle,
            file_pointer,
            product_kind,
        })
    }

    pub fn new_from_file(file_name: String, product_kind: ProductKind) -> Result<Self, CodesError> {
        let product_kind = match_product_kind(product_kind);

        let file_pointer = open_with_fopen(file_name.clone())?;
        let file_handle = CodesHandle::codes_handle_new_from_file(file_pointer, product_kind)?;

        Ok(CodesHandle {
            data: (DataContainer::FileBuffer(file_name)),
            file_handle,
            file_pointer,
            product_kind,
        })
    }

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

impl Drop for CodesHandle {
    ///Executes the desctructor for this type ([read more](https://doc.rust-lang.org/1.54.0/core/ops/drop/trait.Drop.html#tymethod.drop)).
    ///This method calls `codes_handle_delete()` from ecCodes and `fclose()` from libc for graceful cleanup.\
    ///**WARNING:** Currently it is assumed that under normal circumstances this destructor never fails.
    ///However in some edge cases ecCodes or fclose can return non-zero code.
    ///For now user is informed about that through log, because I don't know how to handle it correctly.
    ///If some bugs occurs during drop please enable log output and post issue on Github.
    fn drop(&mut self) {
        let error_code;
        unsafe {
            error_code = eccodes_sys::codes_handle_delete(self.file_handle);
        }

        if error_code != 0 {
            error!(
                "CodesHandle destructor failed with ecCodes error code {:?}",
                error_code
            );
        }

        let error_code;
        unsafe {
            error_code = libc::fclose(self.file_pointer);
        }

        if error_code != 0 {
            let error_val = errno();
            let code = error_val.0;
            error!(
                "CodesHandle destructor failed with libc error {}, code: {}",
                code, error_val
            );
        }

        todo!(
            "Review required! Destructor is assumed to never fail under normal circumstances. 
        However in some edge cases ecCodes or fclose can return non-zero code. 
        For now user is informed about that through log, 
        because I don't know how to handle it correctly."
        );
    }
}

fn match_product_kind(product_kind: ProductKind) -> u32 {
    let product_kind = match product_kind {
        ProductKind::GRIB => ProductKind_PRODUCT_GRIB,
    };

    product_kind
}

fn open_with_fmemopen(file_data: &Bytes) -> Result<*mut FILE, LibcError> {
    let file_size = file_data.len() as size_t;
    let open_mode = "r".as_ptr().cast::<c_char>();

    let file_ptr = file_data.as_ptr() as *mut c_void;

    let file_obj;
    unsafe {
        file_obj = libc::fmemopen(file_ptr, file_size, open_mode);
    }

    if file_obj.is_null() {
        let error_val = errno();
        let error_code = error_val.0;
        return Err(LibcError::NullPtr(error_code, error_val));
    }

    Ok(file_obj)
}

fn open_with_fopen(file_name: String) -> Result<*mut FILE, LibcError> {
    let open_mode = "r".as_ptr().cast::<c_char>();
    let file_name = CString::new(file_name)?;
    let filename_ptr = file_name.as_ptr();

    let file_obj;
    unsafe {
        file_obj = libc::fopen(filename_ptr, open_mode);
    }

    if file_obj.is_null() {
        let error_val = errno();
        let error_code = error_val.0;
        return Err(LibcError::NullPtr(error_code, error_val));
    }

    Ok(file_obj)
}
