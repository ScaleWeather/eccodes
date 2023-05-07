//!Module containing intermediate (type) bindings to ecCodes functions.
//!
//!These bindings convert Rust types to correct C types
//!correctly represent data as pointers and utilize some other functions
//!to make ecCodes usage safer and easier,
//!but they are unsafe as they operate on raw `codes_handle`.  

use std::{
    ffi::{CStr, CString},
    ptr::{self, addr_of_mut},
};

use eccodes_sys::{
    codes_context, codes_handle, codes_keys_iterator, codes_nearest, CODES_NEAREST_SAME_DATA,
    CODES_NEAREST_SAME_GRID, CODES_TYPE_BYTES, CODES_TYPE_DOUBLE, CODES_TYPE_LABEL,
    CODES_TYPE_LONG, CODES_TYPE_MISSING, CODES_TYPE_SECTION, CODES_TYPE_STRING,
    CODES_TYPE_UNDEFINED, _IO_FILE,
};
use libc::{c_void, FILE};
use num_traits::FromPrimitive;

use crate::{
    codes_handle::{NearestGridpoint, ProductKind},
    errors::{CodesError, CodesInternal},
};

#[derive(Copy, Eq, PartialEq, Clone, Ord, PartialOrd, Hash, Debug, num_derive::FromPrimitive)]
pub enum NativeKeyType {
    Undefined = CODES_TYPE_UNDEFINED as isize,
    Long = CODES_TYPE_LONG as isize,
    Double = CODES_TYPE_DOUBLE as isize,
    Str = CODES_TYPE_STRING as isize,
    Bytes = CODES_TYPE_BYTES as isize,
    Section = CODES_TYPE_SECTION as isize,
    Label = CODES_TYPE_LABEL as isize,
    Missing = CODES_TYPE_MISSING as isize,
}

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
        &mut error_code,
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

pub unsafe fn codes_get_native_type(
    handle: *mut codes_handle,
    key: &str,
) -> Result<NativeKeyType, CodesError> {
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
    let buffer_size = codes_get_message_size(handle)?;

    let buffer: Vec<u8> = vec![0; buffer_size];
    let mut buffer_ptr = buffer.as_ptr().cast::<libc::c_void>();

    let mut message_size: usize = 0;

    let error_code = eccodes_sys::codes_get_message(handle, &mut buffer_ptr, &mut message_size);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    assert!(buffer_size == message_size, 
        "Buffer and message sizes ar not equal in codes_get_message! 
        Please report this panic on Github.");

    Ok((buffer_ptr, message_size))
}

pub unsafe fn codes_get_message_copy(handle: *mut codes_handle) -> Result<Vec<u8>, CodesError> {
    let buffer_size = codes_get_message_size(handle)?;

    let mut buffer: Vec<u8> = vec![0; buffer_size];

    let mut message_size = buffer_size;

    let error_code = eccodes_sys::codes_get_message_copy(
        handle,
        buffer.as_mut_ptr().cast::<libc::c_void>(),
        &mut message_size,
    );

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    assert!((buffer_size == message_size && message_size == buffer.len()), 
        "Buffer, vector and message sizes ar not equal in codes_get_message! 
        Please report this panic on Github.");

    Ok(buffer)
}

pub unsafe fn codes_handle_new_from_message_copy(message_buffer: &[u8]) -> *mut codes_handle {
    let default_context: *mut codes_context = ptr::null_mut();

    eccodes_sys::codes_handle_new_from_message_copy(
        default_context,
        message_buffer.as_ptr().cast::<libc::c_void>(),
        message_buffer.len(),
    )
}

pub unsafe fn codes_keys_iterator_new(
    handle: *mut codes_handle,
    flags: u32,
    namespace: &str,
) -> *mut codes_keys_iterator {
    let namespace = CString::new(namespace).unwrap();

    eccodes_sys::codes_keys_iterator_new(handle, u64::from(flags), namespace.as_ptr())
}

pub unsafe fn codes_keys_iterator_delete(
    keys_iterator: *mut codes_keys_iterator,
) -> Result<(), CodesError> {
    let error_code = eccodes_sys::codes_keys_iterator_delete(keys_iterator);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(())
}

pub unsafe fn codes_keys_iterator_next(keys_iterator: *mut codes_keys_iterator) -> bool {
    let next_item_exists = eccodes_sys::codes_keys_iterator_next(keys_iterator);

    next_item_exists == 1
}

pub unsafe fn codes_keys_iterator_get_name(
    keys_iterator: *mut codes_keys_iterator,
) -> Result<String, CodesError> {
    let name_pointer = eccodes_sys::codes_keys_iterator_get_name(keys_iterator);

    let name_c_str = CStr::from_ptr(name_pointer);
    let name_str = name_c_str.to_str()?;
    let name_string = name_str.to_owned();

    Ok(name_string)
}

pub unsafe fn codes_grib_nearest_new(
    handle: *mut codes_handle,
) -> Result<*mut codes_nearest, CodesError> {
    let mut error_code: i32 = 0;

    let nearest = eccodes_sys::codes_grib_nearest_new(handle, &mut error_code);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(nearest)
}

pub unsafe fn codes_grib_nearest_delete(nearest: *mut codes_nearest) -> Result<(), CodesError> {
    let error_code = eccodes_sys::codes_grib_nearest_delete(nearest);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(())
}

pub unsafe fn codes_grib_nearest_find(
    handle: *mut codes_handle,
    nearest: *mut codes_nearest,
    lat: f64,
    lon: f64,
) -> Result<[NearestGridpoint; 4], CodesError> {
    // such flags are set because find nearest for given nearest is always
    // called on the same grib message
    let flags = CODES_NEAREST_SAME_GRID + CODES_NEAREST_SAME_DATA;

    let mut output_lats = [0_f64; 4];
    let mut output_lons = [0_f64; 4];
    let mut output_values = [0_f64; 4];
    let mut output_distances = [0_f64; 4];
    let mut output_indexes = [0_i32; 4];

    let mut length: usize = 4;

    let error_code = eccodes_sys::codes_grib_nearest_find(
        nearest,
        handle,
        lat,
        lon,
        u64::from(flags),
        addr_of_mut!(output_lats[0]),
        addr_of_mut!(output_lons[0]),
        addr_of_mut!(output_values[0]),
        addr_of_mut!(output_distances[0]),
        addr_of_mut!(output_indexes[0]),
        &mut length,
    );

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    let mut output = [NearestGridpoint::default(); 4];

    for i in 0..4 {
        output[i].lat = output_lats[i];
        output[i].lon = output_lons[i];
        output[i].distance = output_distances[i];
        output[i].index = output_indexes[i];
        output[i].value = output_values[i];
    }

    Ok(output)
}

pub unsafe fn codes_set_long(
    handle: *mut codes_handle,
    key: &str,
    value: i64,
) -> Result<(), CodesError> {
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
    let key = CString::new(key).unwrap();

    let length = values.len();

    let error_code = eccodes_sys::codes_set_long_array(
        handle,
        key.as_ptr(),
        values.as_ptr().cast::<i64>(),
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
    let key = CString::new(key).unwrap();

    let length = values.len();

    let error_code = eccodes_sys::codes_set_double_array(
        handle,
        key.as_ptr(),
        values.as_ptr().cast::<f64>(),
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
    let key = CString::new(key).unwrap();

    let mut length = values.len();

    let error_code = eccodes_sys::codes_set_bytes(
        handle,
        key.as_ptr(),
        values.as_ptr().cast::<u8>(),
        &mut length,
    );

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(())
}
