use std::cmp::Ordering;

use crate::{
    CodesError,
    atomic_message::AtomicMessage,
    codes_handle::ThreadSafeHandle,
    intermediate_bindings::{
        NativeKeyType, codes_get_bytes, codes_get_double, codes_get_double_array, codes_get_long,
        codes_get_long_array, codes_get_native_type, codes_get_size, codes_get_string,
    },
};

#[doc(hidden)]
pub trait KeyReadHelpers {
    fn get_key_size(&mut self, key_name: &str) -> Result<usize, CodesError>;
    fn get_key_native_type(&mut self, key_name: &str) -> Result<NativeKeyType, CodesError>;
}
pub trait KeyRead<T>: KeyReadHelpers {
    fn read_key_unchecked(&mut self, name: &str) -> Result<T, CodesError>;
}

pub trait ArrayKeyRead<T>: KeyRead<T> {
    fn read_key(&mut self, key_name: &str) -> Result<T, CodesError> {
        match self.get_key_native_type(key_name)? {
            NativeKeyType::Bytes => (),
            _ => return Err(CodesError::WrongRequestedKeyType),
        }

        let key_size = self.get_key_size(key_name)?;

        if key_size < 1 {
            return Err(CodesError::IncorrectKeySize);
        }

        self.read_key_unchecked(key_name)
    }
}

pub trait ScalarKeyRead<T>: KeyRead<T> {
    fn read_key(&mut self, key_name: &str) -> Result<T, CodesError> {
        match self.get_key_native_type(key_name)? {
            NativeKeyType::Long => (),
            _ => return Err(CodesError::WrongRequestedKeyType),
        }

        let key_size = self.get_key_size(key_name)?;

        match key_size.cmp(&1) {
            Ordering::Greater => return Err(CodesError::WrongRequestedKeySize),
            Ordering::Less => return Err(CodesError::IncorrectKeySize),
            Ordering::Equal => (),
        }

        self.read_key_unchecked(key_name)
    }
}

impl<S: ThreadSafeHandle> KeyReadHelpers for AtomicMessage<S> {
    fn get_key_size(&mut self, key_name: &str) -> Result<usize, CodesError> {
        unsafe { codes_get_size(self.message_handle, key_name) }
    }

    fn get_key_native_type(&mut self, key_name: &str) -> Result<NativeKeyType, CodesError> {
        unsafe { codes_get_native_type(self.message_handle, key_name) }
    }
}

impl<S: ThreadSafeHandle> KeyRead<i64> for AtomicMessage<S> {
    fn read_key_unchecked(&mut self, key_name: &str) -> Result<i64, CodesError> {
        unsafe { codes_get_long(self.message_handle, key_name) }
    }
}

impl<S: ThreadSafeHandle> KeyRead<f64> for AtomicMessage<S> {
    fn read_key_unchecked(&mut self, key_name: &str) -> Result<f64, CodesError> {
        unsafe { codes_get_double(self.message_handle, key_name) }
    }
}

impl<S: ThreadSafeHandle> KeyRead<String> for AtomicMessage<S> {
    fn read_key_unchecked(&mut self, key_name: &str) -> Result<String, CodesError> {
        unsafe { codes_get_string(self.message_handle, key_name) }
    }
}

impl<S: ThreadSafeHandle> KeyRead<Vec<i64>> for AtomicMessage<S> {
    fn read_key_unchecked(&mut self, key_name: &str) -> Result<Vec<i64>, CodesError> {
        unsafe { codes_get_long_array(self.message_handle, key_name) }
    }
}

impl<S: ThreadSafeHandle> KeyRead<Vec<f64>> for AtomicMessage<S> {
    fn read_key_unchecked(&mut self, key_name: &str) -> Result<Vec<f64>, CodesError> {
        unsafe { codes_get_double_array(self.message_handle, key_name) }
    }
}

impl<S: ThreadSafeHandle> KeyRead<Vec<u8>> for AtomicMessage<S> {
    fn read_key_unchecked(&mut self, key_name: &str) -> Result<Vec<u8>, CodesError> {
        unsafe { codes_get_bytes(self.message_handle, key_name) }
    }
}

impl<S: ThreadSafeHandle> ScalarKeyRead<i64> for AtomicMessage<S> {}
impl<S: ThreadSafeHandle> ScalarKeyRead<f64> for AtomicMessage<S> {}

impl<S: ThreadSafeHandle> ArrayKeyRead<String> for AtomicMessage<S> {}
impl<S: ThreadSafeHandle> ArrayKeyRead<Vec<f64>> for AtomicMessage<S> {}
impl<S: ThreadSafeHandle> ArrayKeyRead<Vec<i64>> for AtomicMessage<S> {}
impl<S: ThreadSafeHandle> ArrayKeyRead<Vec<u8>> for AtomicMessage<S> {}
