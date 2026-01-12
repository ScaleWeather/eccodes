#![allow(non_camel_case_types)]
#![allow(clippy::module_name_repetitions)]

use std::ptr::{self};

use eccodes_sys::{codes_context, codes_handle};
use libc::FILE;
use tracing::instrument;

use crate::{
    codes_file::ProductKind, errors::CodesError, intermediate_bindings::error_code_to_result,
    pointer_guard,
};

#[instrument(level = "trace")]
pub unsafe fn codes_handle_new_from_file(
    file_pointer: *mut FILE,
    product_kind: ProductKind,
) -> Result<*mut codes_handle, CodesError> {
    unsafe {
        pointer_guard::non_null!(file_pointer);

        let context: *mut codes_context = ptr::null_mut(); //default context

        let mut error_code: i32 = 0;

        let file_handle = eccodes_sys::codes_handle_new_from_file(
            context,
            file_pointer.cast(),
            product_kind as u32,
            &raw mut error_code,
        );
        error_code_to_result(error_code)?;

        Ok(file_handle)
    }
}

#[instrument(level = "trace")]
pub unsafe fn codes_handle_delete(handle: *mut codes_handle) -> Result<(), CodesError> {
    unsafe {
        if handle.is_null() {
            return Ok(());
        }

        let error_code = eccodes_sys::codes_handle_delete(handle);
        error_code_to_result(error_code)?;

        Ok(())
    }
}

pub unsafe fn codes_handle_clone(
    source_handle: *const codes_handle,
) -> Result<*mut codes_handle, CodesError> {
    pointer_guard::non_null!(source_handle);

    let clone_handle = unsafe { eccodes_sys::codes_handle_clone(source_handle) };

    if clone_handle.is_null() {
        return Err(CodesError::CloneFailed);
    }

    Ok(clone_handle)
}
