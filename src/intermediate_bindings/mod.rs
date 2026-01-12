#![allow(non_camel_case_types)]

//!Module containing intermediate (type) bindings to ecCodes functions.
//!
//!These bindings convert Rust types to correct C types
//!correctly represent data as pointers and utilize some other functions
//!to make ecCodes usage safer and easier,
//!but they are unsafe as they operate on raw `codes_handle`.  

mod codes_get;
mod codes_handle;
mod codes_keys;
mod codes_set;
mod grib_nearest;

#[derive(Copy, Eq, PartialEq, Clone, Ord, PartialOrd, Hash, Debug, num_derive::FromPrimitive)]
pub enum NativeKeyType {
    Undefined = eccodes_sys::CODES_TYPE_UNDEFINED as isize,
    Long = eccodes_sys::CODES_TYPE_LONG as isize,
    Double = eccodes_sys::CODES_TYPE_DOUBLE as isize,
    Str = eccodes_sys::CODES_TYPE_STRING as isize,
    Bytes = eccodes_sys::CODES_TYPE_BYTES as isize,
    Section = eccodes_sys::CODES_TYPE_SECTION as isize,
    Label = eccodes_sys::CODES_TYPE_LABEL as isize,
    Missing = eccodes_sys::CODES_TYPE_MISSING as isize,
}

pub use codes_get::{
    codes_get_bytes, codes_get_double, codes_get_double_array, codes_get_long,
    codes_get_long_array, codes_get_message, codes_get_native_type, codes_get_size,
    codes_get_string,
};
pub use codes_handle::{
    codes_handle_clone, codes_handle_delete, codes_handle_new_from_file,
};
pub use codes_keys::{
    codes_keys_iterator_delete, codes_keys_iterator_get_name, codes_keys_iterator_new,
    codes_keys_iterator_next,
};
pub use codes_set::{
    codes_set_bytes, codes_set_double, codes_set_double_array, codes_set_long,
    codes_set_long_array, codes_set_string,
};
pub use grib_nearest::{
    codes_grib_nearest_delete, codes_grib_nearest_find, codes_grib_nearest_new,
};
use num_traits::FromPrimitive;

use crate::{CodesError, errors::CodesInternal};

fn error_code_to_result(error_code: i32) -> Result<(), CodesError> {
    if error_code != 0 {
        let err: CodesInternal = FromPrimitive::from_i32(error_code)
            .ok_or(CodesError::UnrecognizedErrorCode(error_code))?;
        return Err(err.into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{CodesError, errors::CodesInternal, intermediate_bindings::error_code_to_result};

    #[test]
    fn error_codes() {
        let ok_result = error_code_to_result(0);
        let defined_error = error_code_to_result(-2).unwrap_err(); // codes internal
        let undefined_error = error_code_to_result(4).unwrap_err();

        assert!(ok_result.is_ok());
        assert!(std::matches!(
            defined_error,
            CodesError::Internal(CodesInternal::CodesInternalError)
        ));
        assert!(std::matches!(
            undefined_error,
            CodesError::UnrecognizedErrorCode(4)
        ));
    }
}
