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
    fn read_key(&mut self, key_name: &str) -> Result<T, CodesError>;
    fn read_key_unchecked(&mut self, key_name: &str) -> Result<T, CodesError>;
}

macro_rules! impl_key_read {
    ($key_sizing:ident, $ec_func:ident, $key_variant:path, $gen_type:ty) => {
        impl<S: ThreadSafeHandle> KeyRead<$gen_type> for AtomicMessage<S> {
            fn read_key_unchecked(&mut self, key_name: &str) -> Result<$gen_type, CodesError> {
                unsafe { $ec_func(self.message_handle, key_name) }
            }

            fn read_key(&mut self, key_name: &str) -> Result<$gen_type, CodesError> {
                match self.get_key_native_type(key_name)? {
                    $key_variant => (),
                    _ => return Err(CodesError::WrongRequestedKeyType),
                }

                let key_size = self.get_key_size(key_name)?;

                key_size_check!($key_sizing, key_size);

                self.read_key_unchecked(key_name)
            }
        }
    };
}

macro_rules! key_size_check {
    // size_var is needed because of macro hygiene
    (scalar, $size_var:ident) => {
        match $size_var.cmp(&1) {
            Ordering::Greater => return Err(CodesError::WrongRequestedKeySize),
            Ordering::Less => return Err(CodesError::IncorrectKeySize),
            Ordering::Equal => (),
        }
    };

    (array, $size_var:ident) => {
        if $size_var < 1 {
            return Err(CodesError::IncorrectKeySize);
        }
    };
}

impl<S: ThreadSafeHandle> KeyReadHelpers for AtomicMessage<S> {
    fn get_key_size(&mut self, key_name: &str) -> Result<usize, CodesError> {
        unsafe { codes_get_size(self.message_handle, key_name) }
    }

    fn get_key_native_type(&mut self, key_name: &str) -> Result<NativeKeyType, CodesError> {
        unsafe { codes_get_native_type(self.message_handle, key_name) }
    }
}

impl_key_read!(scalar, codes_get_long, NativeKeyType::Long, i64);
impl_key_read!(scalar, codes_get_double, NativeKeyType::Double, f64);
impl_key_read!(array, codes_get_string, NativeKeyType::Str, String);
impl_key_read!(array, codes_get_bytes, NativeKeyType::Bytes, Vec<u8>);
impl_key_read!(array, codes_get_long_array, NativeKeyType::Long, Vec<i64>);
impl_key_read!(
    array,
    codes_get_double_array,
    NativeKeyType::Double,
    Vec<f64>
);
