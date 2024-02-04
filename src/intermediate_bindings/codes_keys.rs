use std::ffi::{CStr, CString};

use eccodes_sys::{codes_handle, codes_keys_iterator};

use num_traits::FromPrimitive;

use crate::{
    errors::{CodesError, CodesInternal},
    pointer_guard,
};

pub unsafe fn codes_keys_iterator_new(
    handle: *mut codes_handle,
    flags: u32,
    namespace: &str,
) -> Result<*mut codes_keys_iterator, CodesError> {
    pointer_guard::non_null!(handle);

    let namespace = CString::new(namespace).unwrap();

    let kiter = eccodes_sys::codes_keys_iterator_new(handle, u64::from(flags), namespace.as_ptr());

    if kiter.is_null() {
        return Err(CodesError::KeysIteratorFailed);
    }

    Ok(kiter)
}

pub unsafe fn codes_keys_iterator_delete(
    keys_iterator: *mut codes_keys_iterator,
) -> Result<(), CodesError> {
    if keys_iterator.is_null() {
        return Ok(());
    }

    let error_code = eccodes_sys::codes_keys_iterator_delete(keys_iterator);

    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code).unwrap();
        return Err(err.into());
    }

    Ok(())
}

pub unsafe fn codes_keys_iterator_next(
    keys_iterator: *mut codes_keys_iterator,
) -> Result<bool, CodesError> {
    pointer_guard::non_null!(keys_iterator);

    let next_item_exists = eccodes_sys::codes_keys_iterator_next(keys_iterator);

    Ok(next_item_exists == 1)
}

pub unsafe fn codes_keys_iterator_get_name(
    keys_iterator: *mut codes_keys_iterator,
) -> Result<String, CodesError> {
    pointer_guard::non_null!(keys_iterator);

    let name_pointer = eccodes_sys::codes_keys_iterator_get_name(keys_iterator);

    let name_c_str = CStr::from_ptr(name_pointer);
    let name_str = name_c_str.to_str()?;
    let name_string = name_str.to_owned();

    Ok(name_string)
}
