//!Module containing intermediate (type) bindings to ecCodes functions.
//!
//!These bindings convert Rust types to correct C types
//!correctly represent data as pointers and utilize some other functions
//!to make ecCodes usage safer and easier,
//!but they are unsafe as they operate on raw `codes_handle`.  

use std::{
    ffi::{CStr, CString},
    ptr,
};

use eccodes_sys::{codes_context, codes_handle, _IO_FILE};
use libc::{c_void, FILE};
use num_traits::FromPrimitive;

use crate::{
    codes_handle::ProductKind,
    errors::{CodesError, CodesInternal},
};

pub unsafe fn codes_handle_new_from_file(
    file_pointer: *mut FILE,
    product_kind: ProductKind,
) -> Result<*mut codes_handle, CodesError> {
    let context: *mut codes_context = ptr::null_mut(); //default context

    let mut error_code: i32 = 0;

    let file_handle = eccodes_sys::codes_handle_new_from_file(
        context,
        file_pointer.cast::<_IO_FILE>(),
        product_kind as u32,
        &mut error_code as *mut i32,
    );

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(file_handle)
}

pub unsafe fn codes_handle_delete(handle: *mut codes_handle) -> Result<(), CodesError> {
    let error_code = eccodes_sys::codes_handle_delete(handle);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(())
}

#[derive(Copy, Eq, PartialEq, Clone, Ord, PartialOrd, Hash, Debug, num_derive::FromPrimitive)]
pub enum NativeKeyType {
    Long = 1,
    Double = 2,
    Str = 3,
}

pub unsafe fn codes_get_native_type(
    handle: *mut codes_handle,
    key: &str,
) -> Result<NativeKeyType, CodesError> {
    let key = CString::new(key).unwrap();
    let mut key_type: i32 = 0;

    let error_code =
        eccodes_sys::codes_get_native_type(handle, key.as_ptr(), &mut key_type as *mut i32);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(FromPrimitive::from_i32(key_type).unwrap())
}

pub unsafe fn codes_get_size(handle: *mut codes_handle, key: &str) -> Result<u64, CodesError> {
    let key = CString::new(key).unwrap();
    let mut key_size: u64 = 0;

    let error_code = eccodes_sys::codes_get_size(handle, key.as_ptr(), &mut key_size as *mut u64);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(key_size)
}

pub unsafe fn codes_get_long(handle: *mut codes_handle, key: &str) -> Result<i64, CodesError> {
    let key = CString::new(key).unwrap();
    let mut key_value: i64 = 0;

    let error_code = eccodes_sys::codes_get_long(handle, key.as_ptr(), &mut key_value as *mut i64);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(key_value)
}

pub unsafe fn codes_get_double(handle: *mut codes_handle, key: &str) -> Result<f64, CodesError> {
    let key = CString::new(key).unwrap();
    let mut key_value: f64 = 0.0;

    let error_code =
        eccodes_sys::codes_get_double(handle, key.as_ptr(), &mut key_value as *mut f64);

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
    let mut key_size = codes_get_size(handle, key)?;
    let key = CString::new(key).unwrap();

    let mut key_values: Vec<f64> = vec![0.0; key_size as usize];

    let error_code = eccodes_sys::codes_get_double_array(
        handle,
        key.as_ptr(),
        key_values.as_mut_ptr().cast::<f64>(),
        &mut key_size as *mut u64,
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
    let mut key_size = codes_get_size(handle, key)?;
    let key = CString::new(key).unwrap();

    let mut key_values: Vec<i64> = vec![0; key_size as usize];

    let error_code = eccodes_sys::codes_get_long_array(
        handle,
        key.as_ptr(),
        key_values.as_mut_ptr().cast::<i64>(),
        &mut key_size as *mut u64,
    );

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(key_values)
}

pub unsafe fn codes_get_length(handle: *mut codes_handle, key: &str) -> Result<u64, CodesError> {
    let key = CString::new(key).unwrap();
    let mut key_length: u64 = 0;

    let error_code =
        eccodes_sys::codes_get_length(handle, key.as_ptr(), &mut key_length as *mut u64);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(key_length)
}

pub unsafe fn codes_get_string(handle: *mut codes_handle, key: &str) -> Result<String, CodesError> {
    let mut key_length = codes_get_length(handle, key)?;
    let key = CString::new(key).unwrap();

    let mut key_message: Vec<u8> = vec![0; key_length as usize];

    let error_code = eccodes_sys::codes_get_string(
        handle,
        key.as_ptr(),
        key_message.as_mut_ptr().cast::<i8>(),
        &mut key_length as *mut u64,
    );

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    key_message.truncate(key_length as usize);
    let key_message = CStr::from_bytes_with_nul(key_message.as_ref())?
        .to_str()?
        .to_string();

    Ok(key_message)
}

pub unsafe fn codes_get_message_size(handle: *mut codes_handle) -> Result<u64, CodesError> {
    let mut size: u64 = 0;

    let error_code = eccodes_sys::codes_get_message_size(handle, &mut size as *mut u64);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(size)
}

pub unsafe fn codes_get_message(handle: *mut codes_handle) -> Result<(*const c_void, u64), CodesError> {
    let buffer_size = codes_get_message_size(handle)?;

    let buffer: Vec<u8> = vec![0; buffer_size as usize];
    let mut buffer_ptr = buffer.as_ptr() as *const c_void;

    let mut message_size: u64 = 0;

    let error_code = eccodes_sys::codes_get_message(
        handle,
        &mut buffer_ptr as *mut *const c_void,
        &mut message_size as *mut u64,
    );

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    if buffer_size != message_size {
        panic!(
            "Buffer and message sizes ar not equal in codes_get_message!
        Please report this panic on Github."
        );
    }

    Ok((buffer_ptr, message_size))
}

pub unsafe fn codes_handle_new_from_message(
    message_buffer_ptr: *const c_void,
    message_size: u64,
) -> *mut codes_handle {
    let default_context: *mut codes_context = ptr::null_mut();

    let handle = eccodes_sys::codes_handle_new_from_message(
        default_context,
        message_buffer_ptr,
        message_size,
    );

    handle
}
