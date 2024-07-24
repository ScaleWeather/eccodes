use eccodes_sys::grib_handle;

use crate::{
    intermediate_bindings::{
        codes_get_bytes, codes_get_double, codes_get_double_array, codes_get_long,
        codes_get_long_array, codes_get_native_type, codes_get_size, codes_get_string,
        NativeKeyType,
    },
    CodesError, Key, KeyOps, KeyedMessage,
};

impl KeyedMessage {
    fn get_key_size(&self, key_name: &str) -> Result<usize, CodesError> {
        unsafe { codes_get_size(self.message_handle, key_name) }
    }

    fn get_key_native_type(&self, key_name: &str) -> Result<NativeKeyType, CodesError> {
        unsafe { codes_get_native_type(self.message_handle, key_name) }
    }
}

impl KeyOps<i64> for KeyedMessage {
    fn read(&self, key_name: &str) -> Result<i64, CodesError> {
        match self.get_key_native_type(key_name)? {
            NativeKeyType::Long => (),
            _ => return Err(CodesError::WrongRequestedKeyType),
        }

        let key_size = self.get_key_size(key_name)?;

        if key_size < 1 {
            return Err(CodesError::IncorrectKeySize);
        } else if key_size > 1 {
            return Err(CodesError::WrongRequestedKeySize);
        }

        todo!()
    }

    fn read_unchecked(&self, key_name: &str) -> Result<i64, CodesError> {
        unsafe { codes_get_long(self.message_handle, key_name) }
    }

    fn write(&mut self, key: Key<i64>) -> Result<(), CodesError> {
        todo!()
    }

    fn write_unchecked(&mut self, key: Key<i64>) -> Result<(), CodesError> {
        todo!()
    }
}
