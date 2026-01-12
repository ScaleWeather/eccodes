#![allow(non_camel_case_types)]
#![allow(clippy::module_name_repetitions)]

use std::ffi::{CStr, CString};

use eccodes_sys::codes_handle;
use libc::c_void;
use num_traits::FromPrimitive;

use crate::{errors::CodesError, intermediate_bindings::error_code_to_result, pointer_guard};

use super::NativeKeyType;

pub unsafe fn codes_get_native_type(
    handle: *const codes_handle,
    key: &str,
) -> Result<NativeKeyType, CodesError> {
    unsafe {
        pointer_guard::non_null!(handle);

        let key = CString::new(key).unwrap();
        let mut key_type: i32 = 0;

        let error_code =
            eccodes_sys::codes_get_native_type(handle, key.as_ptr(), &raw mut key_type);
        error_code_to_result(error_code)?;

        FromPrimitive::from_i32(key_type).ok_or(CodesError::UnrecognizedKeyTypeCode(key_type))
    }
}

pub unsafe fn codes_get_size(handle: *const codes_handle, key: &str) -> Result<usize, CodesError> {
    unsafe {
        pointer_guard::non_null!(handle);

        let key = CString::new(key).unwrap();
        let mut key_size: usize = 0;

        let error_code = eccodes_sys::codes_get_size(handle, key.as_ptr(), &raw mut key_size);
        error_code_to_result(error_code)?;

        Ok(key_size)
    }
}

pub unsafe fn codes_get_long(handle: *const codes_handle, key: &str) -> Result<i64, CodesError> {
    unsafe {
        pointer_guard::non_null!(handle);

        let key = CString::new(key).unwrap();
        let mut key_value: i64 = 0;

        let error_code = eccodes_sys::codes_get_long(handle, key.as_ptr(), &raw mut key_value);
        error_code_to_result(error_code)?;

        Ok(key_value)
    }
}

pub unsafe fn codes_get_double(handle: *const codes_handle, key: &str) -> Result<f64, CodesError> {
    unsafe {
        pointer_guard::non_null!(handle);

        let key = CString::new(key).unwrap();
        let mut key_value: f64 = 0.0;

        let error_code = eccodes_sys::codes_get_double(handle, key.as_ptr(), &raw mut key_value);
        error_code_to_result(error_code)?;

        Ok(key_value)
    }
}

pub unsafe fn codes_get_double_array(
    handle: *const codes_handle,
    key: &str,
) -> Result<Vec<f64>, CodesError> {
    unsafe {
        pointer_guard::non_null!(handle);

        let mut key_size = codes_get_size(handle, key)?;
        let key = CString::new(key).unwrap();

        let mut key_values: Vec<f64> = vec![0.0; key_size];

        let error_code = eccodes_sys::codes_get_double_array(
            handle,
            key.as_ptr(),
            key_values.as_mut_ptr().cast(),
            &raw mut key_size,
        );
        error_code_to_result(error_code)?;

        Ok(key_values)
    }
}

pub unsafe fn codes_get_long_array(
    handle: *const codes_handle,
    key: &str,
) -> Result<Vec<i64>, CodesError> {
    unsafe {
        pointer_guard::non_null!(handle);

        let mut key_size = codes_get_size(handle, key)?;
        let key = CString::new(key).unwrap();

        let mut key_values: Vec<i64> = vec![0; key_size];

        let error_code = eccodes_sys::codes_get_long_array(
            handle,
            key.as_ptr(),
            key_values.as_mut_ptr().cast(),
            &raw mut key_size,
        );
        error_code_to_result(error_code)?;

        Ok(key_values)
    }
}

pub unsafe fn codes_get_length(
    handle: *const codes_handle,
    key: &str,
) -> Result<usize, CodesError> {
    unsafe {
        pointer_guard::non_null!(handle);

        let key = CString::new(key).unwrap();
        let mut key_length: usize = 0;

        let error_code = eccodes_sys::codes_get_length(handle, key.as_ptr(), &raw mut key_length);
        error_code_to_result(error_code)?;

        Ok(key_length)
    }
}

pub unsafe fn codes_get_string(
    handle: *const codes_handle,
    key: &str,
) -> Result<String, CodesError> {
    unsafe {
        pointer_guard::non_null!(handle);

        let mut key_length = codes_get_length(handle, key)?;
        let key = CString::new(key).unwrap();

        let mut key_message: Vec<u8> = vec![0; key_length];

        let error_code = eccodes_sys::codes_get_string(
            handle,
            key.as_ptr(),
            key_message.as_mut_ptr().cast(),
            &raw mut key_length,
        );
        error_code_to_result(error_code)?;

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
}

pub unsafe fn codes_get_bytes(
    handle: *const codes_handle,
    key: &str,
) -> Result<Vec<u8>, CodesError> {
    unsafe {
        pointer_guard::non_null!(handle);

        let mut key_size = codes_get_length(handle, key)?;
        let key = CString::new(key).unwrap();

        let mut buffer: Vec<u8> = vec![0; key_size];

        let error_code = eccodes_sys::codes_get_bytes(
            handle,
            key.as_ptr(),
            buffer.as_mut_ptr().cast(),
            &raw mut key_size,
        );
        error_code_to_result(error_code)?;

        Ok(buffer)
    }
}

pub unsafe fn codes_get_message_size(handle: *const codes_handle) -> Result<usize, CodesError> {
    unsafe {
        pointer_guard::non_null!(handle);

        let mut size: usize = 0;

        let error_code = eccodes_sys::codes_get_message_size(handle, &raw mut size);
        error_code_to_result(error_code)?;

        Ok(size)
    }
}

/// Can panic in debug
pub unsafe fn codes_get_message(
    handle: *const codes_handle,
) -> Result<(*const c_void, usize), CodesError> {
    unsafe {
        pointer_guard::non_null!(handle);

        let buffer_size = codes_get_message_size(handle)?;

        let buffer: Vec<u8> = vec![0; buffer_size];
        let mut buffer_ptr = buffer.as_ptr().cast();

        let mut message_size: usize = 0;

        let error_code =
            eccodes_sys::codes_get_message(handle, &raw mut buffer_ptr, &raw mut message_size);
        error_code_to_result(error_code)?;

        debug_assert!(
            buffer_size == message_size,
            "Buffer and message sizes ar not equal in codes_get_message! 
        Please report this panic on Github."
        );

        Ok((buffer_ptr, message_size))
    }
}
