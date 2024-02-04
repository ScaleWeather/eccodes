use std::ptr::{self};

use eccodes_sys::{codes_context, codes_handle, codes_index, CODES_LOCK};
use libc::FILE;
use num_traits::FromPrimitive;

use crate::{
    codes_handle::ProductKind,
    errors::{CodesError, CodesInternal},
    pointer_guard,
};

#[cfg(target_os = "macos")]
type _SYS_IO_FILE = eccodes_sys::__sFILE;

#[cfg(not(target_os = "macos"))]
type _SYS_IO_FILE = eccodes_sys::_IO_FILE;

pub unsafe fn codes_handle_new_from_file(
    file_pointer: *mut FILE,
    product_kind: ProductKind,
) -> Result<*mut codes_handle, CodesError> {
    pointer_guard::non_null!(file_pointer);

    let context: *mut codes_context = ptr::null_mut(); //default context

    let mut error_code: i32 = 0;

    let file_handle = eccodes_sys::codes_handle_new_from_file(
        context,
        file_pointer.cast::<_SYS_IO_FILE>(),
        product_kind as u32,
        &mut error_code,
    );

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(file_handle)
}

pub unsafe fn codes_handle_delete(handle: *mut codes_handle) -> Result<(), CodesError> {
    if handle.is_null() {
        return Ok(());
    }

    let error_code = eccodes_sys::codes_handle_delete(handle);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(())
}

pub unsafe fn codes_handle_new_from_index(
    index: *mut codes_index,
) -> Result<*mut codes_handle, CodesError> {
    pointer_guard::non_null!(index);

    let mut error_code: i32 = 0;

    let _g = CODES_LOCK.lock().unwrap();
    let codes_handle = eccodes_sys::codes_handle_new_from_index(index, &mut error_code);

    // special case! codes_handle_new_from_index returns -43 when there are no messages left in the index
    // this is also indicated by a null pointer, which is handled upstream
    if error_code == -43 {
        return Ok(codes_handle);
    }

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }
    Ok(codes_handle)
}

pub unsafe fn codes_handle_clone(
    source_handle: *mut codes_handle,
) -> Result<*mut codes_handle, CodesError> {
    pointer_guard::non_null!(source_handle);

    let clone_handle = unsafe { eccodes_sys::codes_handle_clone(source_handle) };

    if clone_handle.is_null() {
        return Err(CodesError::CloneFailed);
    }

    Ok(clone_handle)
}
