#![allow(non_camel_case_types)]
#![allow(clippy::module_name_repetitions)]

use eccodes_sys::{codes_index, CODES_LOCK};
use std::{ffi::CString, ptr};

#[cfg(target_os = "macos")]
type _SYS_IO_FILE = eccodes_sys::__sFILE;

#[cfg(not(target_os = "macos"))]
type _SYS_IO_FILE = eccodes_sys::_IO_FILE;

use eccodes_sys::{codes_context, codes_handle};
use num_traits::FromPrimitive;

use crate::{errors::{CodesError, CodesInternal}, pointer_guard};

// all index functions are safeguarded by a lock
// because there are random errors appearing when using the index functions concurrently

pub unsafe fn codes_index_new(keys: &str) -> Result<*mut codes_index, CodesError> {
    let context: *mut codes_context = ptr::null_mut(); //default context
    let mut error_code: i32 = 0;
    let keys = CString::new(keys).unwrap();

    let _g = CODES_LOCK.lock().unwrap();
    let codes_index = eccodes_sys::codes_index_new(context, keys.as_ptr(), &mut error_code);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }
    Ok(codes_index)
}

pub unsafe fn codes_index_read(filename: &str) -> Result<*mut codes_index, CodesError> {
    let filename = CString::new(filename).unwrap();
    let context: *mut codes_context = ptr::null_mut(); //default context
    let mut error_code: i32 = 0;

    let _g = CODES_LOCK.lock().unwrap();
    let codes_index = eccodes_sys::codes_index_read(context, filename.as_ptr(), &mut error_code);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }
    Ok(codes_index)
}

pub unsafe fn codes_index_delete(index: *mut codes_index) {
    if index.is_null() {
        return;
    }

    let _g = CODES_LOCK.lock().unwrap();
    eccodes_sys::codes_index_delete(index);
}

pub unsafe fn codes_index_add_file(
    index: *mut codes_index,
    filename: &str,
) -> Result<(), CodesError> {
    pointer_guard::non_null!(index);

    let filename = CString::new(filename).unwrap();

    let _g = CODES_LOCK.lock().unwrap();
    let error_code = eccodes_sys::codes_index_add_file(index, filename.as_ptr());

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }
    Ok(())
}

pub unsafe fn codes_index_select_long(
    index: *mut codes_index,
    key: &str,
    value: i64,
) -> Result<(), CodesError> {
    pointer_guard::non_null!(index);
    
    let key = CString::new(key).unwrap();

    let _g = CODES_LOCK.lock().unwrap();
    let error_code = eccodes_sys::codes_index_select_long(index, key.as_ptr(), value);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }
    Ok(())
}

pub unsafe fn codes_index_select_double(
    index: *mut codes_index,
    key: &str,
    value: f64,
) -> Result<(), CodesError> {
    pointer_guard::non_null!(index);

    let key = CString::new(key).unwrap();

    let _g = CODES_LOCK.lock().unwrap();
    let error_code = eccodes_sys::codes_index_select_double(index, key.as_ptr(), value);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }
    Ok(())
}

pub unsafe fn codes_index_select_string(
    index: *mut codes_index,
    key: &str,
    value: &str,
) -> Result<(), CodesError> {
    pointer_guard::non_null!(index);

    let key = CString::new(key).unwrap();
    let value = CString::new(value).unwrap();

    let _g = CODES_LOCK.lock().unwrap();
    let error_code = eccodes_sys::codes_index_select_string(index, key.as_ptr(), value.as_ptr());

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
