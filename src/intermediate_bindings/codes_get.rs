#![allow(non_camel_case_types)]
#![allow(clippy::module_name_repetitions)]

use std::ffi::{CStr, CString};

use eccodes_sys::codes_handle;
use libc::c_void;
use num_traits::FromPrimitive;

use crate::{
    errors::{CodesError, CodesInternal},
    pointer_guard,
};

use super::NativeKeyType;

pub unsafe fn codes_get_native_type(
    handle: *mut codes_handle,
    key: &str,
) -> Result<NativeKeyType, CodesError> {
    pointer_guard::non_null!(handle);

    let key = CString::new(key).unwrap();
    let mut key_type: i32 = 0;

    let error_code = eccodes_sys::codes_get_native_type(handle, key.as_ptr(), &mut key_type);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(FromPrimitive::from_i32(key_type).unwrap())
}

pub unsafe fn codes_get_size(handle: *mut codes_handle, key: &str) -> Result<usize, CodesError> {
    pointer_guard::non_null!(handle);

    let key = CString::new(key).unwrap();
    let mut key_size: usize = 0;

    let error_code = eccodes_sys::codes_get_size(handle, key.as_ptr(), &mut key_size);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(key_size)
}

pub unsafe fn codes_get_long(handle: *mut codes_handle, key: &str) -> Result<i64, CodesError> {
    pointer_guard::non_null!(handle);

    let key = CString::new(key).unwrap();
    let mut key_value: i64 = 0;

    let error_code = eccodes_sys::codes_get_long(handle, key.as_ptr(), &mut key_value);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(key_value)
}

pub unsafe fn codes_get_double(handle: *mut codes_handle, key: &str) -> Result<f64, CodesError> {
    pointer_guard::non_null!(handle);

    let key = CString::new(key).unwrap();
    let mut key_value: f64 = 0.0;

    let error_code = eccodes_sys::codes_get_double(handle, key.as_ptr(), &mut key_value);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(key_value)
}

pub unsafe fn codes_get_double_array(
    handle: *mut codes_handle,
    key: &str,
) -> Result<Vec<f64>, CodesError> {
    pointer_guard::non_null!(handle);

    let mut key_size = codes_get_size(handle, key)?;
    let key = CString::new(key).unwrap();

    let mut key_values: Vec<f64> = vec![0.0; key_size];

    let error_code = eccodes_sys::codes_get_double_array(
        handle,
        key.as_ptr(),
        key_values.as_mut_ptr().cast::<f64>(),
        &mut key_size,
    );

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(key_values)
}

pub unsafe fn codes_get_long_array(
    handle: *mut codes_handle,
    key: &str,
) -> Result<Vec<i64>, CodesError> {
    pointer_guard::non_null!(handle);

    let mut key_size = codes_get_size(handle, key)?;
    let key = CString::new(key).unwrap();

    let mut key_values: Vec<i64> = vec![0; key_size];

    let error_code = eccodes_sys::codes_get_long_array(
        handle,
        key.as_ptr(),
        key_values.as_mut_ptr().cast::<i64>(),
        &mut key_size,
    );

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(key_values)
}

pub unsafe fn codes_get_length(handle: *mut codes_handle, key: &str) -> Result<usize, CodesError> {
    pointer_guard::non_null!(handle);

    let key = CString::new(key).unwrap();
    let mut key_length: usize = 0;

    let error_code = eccodes_sys::codes_get_length(handle, key.as_ptr(), &mut key_length);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(key_length)
}

pub unsafe fn codes_get_string(handle: *mut codes_handle, key: &str) -> Result<String, CodesError> {
    pointer_guard::non_null!(handle);

    let mut key_length = codes_get_length(handle, key)?;
    let key = CString::new(key).unwrap();

    let mut key_message: Vec<u8> = vec![0; key_length];

    let error_code = eccodes_sys::codes_get_string(
        handle,
        key.as_ptr(),
        key_message.as_mut_ptr().cast::<i8>(),
        &mut key_length,
    );

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    key_message.truncate(key_length);
    let key_message_result = CStr::from_bytes_with_nul(key_message.as_ref());

    let key_message_cstr = if let Ok(msg) = key_message_result {
        msg
    } else {
        key_message.push(0);
        CStr::from_bytes_with_nul(key_message.as_ref())?
    };

    let key_message_string = key_message_cstr.to_str()?.to_string();

    Ok(key_message_string)
}

pub unsafe fn codes_get_bytes(handle: *mut codes_handle, key: &str) -> Result<Vec<u8>, CodesError> {
    pointer_guard::non_null!(handle);

    let mut key_size = codes_get_length(handle, key)?;
    let key = CString::new(key).unwrap();

    let mut buffer: Vec<u8> = vec![0; key_size];

    let error_code = eccodes_sys::codes_get_bytes(
        handle,
        key.as_ptr(),
        buffer.as_mut_ptr().cast::<u8>(),
        &mut key_size,
    );

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(buffer)
}

pub unsafe fn codes_get_message_size(handle: *mut codes_handle) -> Result<usize, CodesError> {
    pointer_guard::non_null!(handle);

    let mut size: usize = 0;

    let error_code = eccodes_sys::codes_get_message_size(handle, &mut size);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(size)
}

pub unsafe fn codes_get_message(
    handle: *mut codes_handle,
) -> Result<(*const c_void, usize), CodesError> {
    pointer_guard::non_null!(handle);

    let buffer_size = codes_get_message_size(handle)?;

    let buffer: Vec<u8> = vec![0; buffer_size];
    let mut buffer_ptr = buffer.as_ptr().cast::<libc::c_void>();

    let mut message_size: usize = 0;

    let error_code = eccodes_sys::codes_get_message(handle, &mut buffer_ptr, &mut message_size);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    assert!(
        buffer_size == message_size,
        "Buffer and message sizes ar not equal in codes_get_message! 
        Please report this panic on Github."
    );

    Ok((buffer_ptr, message_size))
}
