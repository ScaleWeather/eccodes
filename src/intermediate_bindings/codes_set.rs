#![allow(non_camel_case_types)]
#![allow(clippy::module_name_repetitions)]

use std::ffi::CString;

use eccodes_sys::codes_handle;

use num_traits::FromPrimitive;

use crate::{
    errors::{CodesError, CodesInternal},
    pointer_guard,
};

pub unsafe fn codes_set_long(
    handle: *mut codes_handle,
    key: &str,
    value: i64,
) -> Result<(), CodesError> {
    pointer_guard::non_null!(handle);

    let key = CString::new(key).unwrap();

    let error_code = eccodes_sys::codes_set_long(handle, key.as_ptr(), value);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(())
}

pub unsafe fn codes_set_double(
    handle: *mut codes_handle,
    key: &str,
    value: f64,
) -> Result<(), CodesError> {
    pointer_guard::non_null!(handle);

    let key = CString::new(key).unwrap();

    let error_code = eccodes_sys::codes_set_double(handle, key.as_ptr(), value);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(())
}

pub unsafe fn codes_set_long_array(
    handle: *mut codes_handle,
    key: &str,
    values: &[i64],
) -> Result<(), CodesError> {
    pointer_guard::non_null!(handle);

    let key = CString::new(key).unwrap();

    let length = values.len();

    let error_code = eccodes_sys::codes_set_long_array(
        handle,
        key.as_ptr(),
        values.as_ptr().cast::<_>(),
        length,
    );

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(())
}

pub unsafe fn codes_set_double_array(
    handle: *mut codes_handle,
    key: &str,
    values: &[f64],
) -> Result<(), CodesError> {
    pointer_guard::non_null!(handle);

    let key = CString::new(key).unwrap();

    let length = values.len();

    let error_code = eccodes_sys::codes_set_double_array(
        handle,
        key.as_ptr(),
        values.as_ptr().cast::<_>(),
        length,
    );

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(())
}

pub unsafe fn codes_set_string(
    handle: *mut codes_handle,
    key: &str,
    value: &str,
) -> Result<(), CodesError> {
    pointer_guard::non_null!(handle);

    let key = CString::new(key).unwrap();
    let mut length = value.len();
    let value = CString::new(value).unwrap();

    let error_code =
        eccodes_sys::codes_set_string(handle, key.as_ptr(), value.as_ptr(), &mut length);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(())
}

pub unsafe fn codes_set_bytes(
    handle: *mut codes_handle,
    key: &str,
    values: &[u8],
) -> Result<(), CodesError> {
    pointer_guard::non_null!(handle);

    let key = CString::new(key).unwrap();

    let mut length = values.len();

    let error_code = eccodes_sys::codes_set_bytes(
        handle,
        key.as_ptr(),
        values.as_ptr().cast::<_>(),
        &mut length,
    );

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(())
}
