macro_rules! non_null {
    ($ptr:expr) => {
        if $ptr.is_null() {
            debug_assert!(false, "Null pointer encountered");
            return Err(CodesError::NullPtr);
        }
    };
}
pub(crate) use non_null;

#[cfg(test)]
mod tests {
    use crate::errors::CodesError;
    use std::ptr;

    #[test]
    #[should_panic = "Null pointer encountered"]
    fn test_non_null() {
        let ptr: *mut i32 = ptr::null_mut();
        let result = simulated_function(ptr);

        assert!(result.is_err());

        let result = result.unwrap_err();

        match result {
            CodesError::NullPtr => (),
            _ => panic!("Incorrect error type: {result:?}"),
        }
    }

    #[test]
    fn test_non_null_ok() {
        let mut x = 42_i32;
        let ptr = &raw mut x;

        let result = simulated_function(ptr);

        assert!(result.is_ok());
    }

    fn simulated_function(ptr: *mut i32) -> Result<(), CodesError> {
        non_null!(ptr);
        Ok(())
    }
}
