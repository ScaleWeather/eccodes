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
    use crate::pointer_guard::non_null;
    use std::ptr;

    #[test]
    #[should_panic]
    fn test_non_null() {
        let ptr: *mut i32 = ptr::null_mut();
        let result = simulated_function(ptr);

        assert!(result.is_err());

        let result = result.unwrap_err();

        match result {
            CodesError::NullPtr => (),
            _ => panic!("Incorrect error type: {:?}", result),
        }
    }

    #[test]
    fn test_non_null_ok() {
        let mut x = 42_i32;
        let ptr = &mut x as *mut i32;

        let result = simulated_function(ptr);

        assert!(result.is_ok());
    }

    fn simulated_function(ptr: *mut i32) -> Result<(), CodesError> {
        non_null!(ptr);
        Ok(())
    }
}
