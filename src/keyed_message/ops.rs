use crate::{
    intermediate_bindings::{
        codes_get_bytes, codes_get_double, codes_get_double_array, codes_get_long,
        codes_get_long_array, codes_get_native_type, codes_get_size, codes_get_string,
        codes_set_bytes, codes_set_double, codes_set_double_array, codes_set_long,
        codes_set_long_array, codes_set_string, NativeKeyType,
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

        self.read_unchecked(key_name)
    }

    fn read_unchecked(&self, key_name: &str) -> Result<i64, CodesError> {
        unsafe { codes_get_long(self.message_handle, key_name) }
    }

    fn write(&mut self, key: Key<i64>) -> Result<(), CodesError> {
        unsafe { codes_set_long(self.message_handle, &key.name, key.value) }
    }
}

impl KeyOps<f64> for KeyedMessage {
    fn read(&self, key_name: &str) -> Result<f64, CodesError> {
        match self.get_key_native_type(key_name)? {
            NativeKeyType::Double => (),
            _ => return Err(CodesError::WrongRequestedKeyType),
        }

        let key_size = self.get_key_size(key_name)?;

        if key_size < 1 {
            return Err(CodesError::IncorrectKeySize);
        } else if key_size > 1 {
            return Err(CodesError::WrongRequestedKeySize);
        }

        self.read_unchecked(key_name)
    }

    fn read_unchecked(&self, key_name: &str) -> Result<f64, CodesError> {
        unsafe { codes_get_double(self.message_handle, key_name) }
    }

    fn write(&mut self, key: Key<f64>) -> Result<(), CodesError> {
        unsafe { codes_set_double(self.message_handle, &key.name, key.value) }
    }
}

impl KeyOps<String> for KeyedMessage {
    fn read(&self, key_name: &str) -> Result<String, CodesError> {
        match self.get_key_native_type(key_name)? {
            NativeKeyType::Str => (),
            _ => return Err(CodesError::WrongRequestedKeyType),
        }

        let key_size = self.get_key_size(key_name)?;

        if key_size < 1 {
            return Err(CodesError::IncorrectKeySize);
        }

        self.read_unchecked(key_name)
    }

    fn read_unchecked(&self, key_name: &str) -> Result<String, CodesError> {
        unsafe { codes_get_string(self.message_handle, key_name) }
    }

    fn write(&mut self, key: Key<String>) -> Result<(), CodesError> {
        unsafe { codes_set_string(self.message_handle, &key.name, &key.value) }
    }
}

impl KeyOps<Vec<i64>> for KeyedMessage {
    fn read(&self, key_name: &str) -> Result<Vec<i64>, CodesError> {
        match self.get_key_native_type(key_name)? {
            NativeKeyType::Long => (),
            _ => return Err(CodesError::WrongRequestedKeyType),
        }

        let key_size = self.get_key_size(key_name)?;

        if key_size < 1 {
            return Err(CodesError::IncorrectKeySize);
        }

        self.read_unchecked(key_name)
    }

    fn read_unchecked(&self, key_name: &str) -> Result<Vec<i64>, CodesError> {
        unsafe { codes_get_long_array(self.message_handle, key_name) }
    }

    fn write(&mut self, key: Key<Vec<i64>>) -> Result<(), CodesError> {
        unsafe { codes_set_long_array(self.message_handle, &key.name, &key.value) }
    }
}

impl KeyOps<Vec<f64>> for KeyedMessage {
    fn read(&self, key_name: &str) -> Result<Vec<f64>, CodesError> {
        match self.get_key_native_type(key_name)? {
            NativeKeyType::Double => (),
            _ => return Err(CodesError::WrongRequestedKeyType),
        }

        let key_size = self.get_key_size(key_name)?;

        if key_size < 1 {
            return Err(CodesError::IncorrectKeySize);
        }

        self.read_unchecked(key_name)
    }

    fn read_unchecked(&self, key_name: &str) -> Result<Vec<f64>, CodesError> {
        unsafe { codes_get_double_array(self.message_handle, key_name) }
    }

    fn write(&mut self, key: Key<Vec<f64>>) -> Result<(), CodesError> {
        unsafe { codes_set_double_array(self.message_handle, &key.name, &key.value) }
    }
}

impl KeyOps<Vec<u8>> for KeyedMessage {
    fn read(&self, key_name: &str) -> Result<Vec<u8>, CodesError> {
        match self.get_key_native_type(key_name)? {
            NativeKeyType::Bytes => (),
            _ => return Err(CodesError::WrongRequestedKeyType),
        }

        let key_size = self.get_key_size(key_name)?;

        if key_size < 1 {
            return Err(CodesError::IncorrectKeySize);
        }

        self.read_unchecked(key_name)
    }

    fn read_unchecked(&self, key_name: &str) -> Result<Vec<u8>, CodesError> {
        unsafe { codes_get_bytes(self.message_handle, key_name) }
    }

    fn write(&mut self, key: Key<Vec<u8>>) -> Result<(), CodesError> {
        unsafe { codes_set_bytes(self.message_handle, &key.name, &key.value) }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use anyhow::{Context, Result};
    use fallible_iterator::FallibleIterator;
    use fallible_streaming_iterator::FallibleStreamingIterator;

    use crate::{CodesError, CodesHandle, KeyOps, ProductKind};
    #[test]
    fn static_read() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        static_read_core(file_path)?;

        let file_path = Path::new("./data/gfs.grib");
        static_read_core(file_path)?;

        Ok(())
    }

    fn static_read_core(file_path: &Path) -> Result<()> {
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.context("Message not some")?;
        let mut kiter = current_message.default_keys_iterator()?;

        while let Some(key_name) = kiter.next()? {
            assert!(!key_name.is_empty());

            // annoying keys
            if ["zero", "zeros"].contains(&key_name.as_str()) {
                continue;
            }

            let kv: Result<i64, CodesError> = current_message.read(&key_name);
            validate_read_error(kv, &key_name)?;

            let kv: Result<f64, CodesError> = current_message.read(&key_name);
            validate_read_error(kv, &key_name)?;

            let kv: Result<String, CodesError> = current_message.read(&key_name);
            validate_read_error(kv, &key_name)?;

            let kv: Result<Vec<i64>, CodesError> = current_message.read(&key_name);
            validate_read_error(kv, &key_name)?;

            let kv: Result<Vec<f64>, CodesError> = current_message.read(&key_name);
            validate_read_error(kv, &key_name)?;

            let kv: Result<Vec<u8>, CodesError> = current_message.read(&key_name);
            validate_read_error(kv, &key_name)?;
        }

        Ok(())
    }

    fn validate_read_error<T>(kv: Result<T, CodesError>, key_name: &str) -> Result<()> {
        let allowed_incorrect_size = [
            "localExtensionPadding",
            "section1Padding",
            "padding_sec2_2",
            "padding_sec2_1",
            "padding_sec2_3",
            "padding_sec4_1",
            "section3Padding",
        ];

        match kv {
            Ok(_) => return Ok(()),
            Err(ke) => match ke {
                CodesError::WrongRequestedKeySize => return Ok(()),
                CodesError::WrongRequestedKeyType => return Ok(()),
                CodesError::IncorrectKeySize => {
                    if allowed_incorrect_size.contains(&&key_name) {
                        return Ok(());
                    }

                    panic!("Key {key_name} not on a list allowed to have incorrect size")
                }
                _ => panic!("Unexpected error {ke} for {key_name}"),
            },
        }
    }
}
