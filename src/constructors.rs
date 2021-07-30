use crate::{CodesError, CodesHandle, DataContainer, ProductKind, LibcError};
use bytes::Bytes;
use eccodes_sys::{self, ProductKind_PRODUCT_GRIB};
use errno::errno;
use libc::{c_char, c_void, size_t, FILE};
use std::ffi::CString;

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
