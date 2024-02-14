use crate::{
    errors::CodesError,
    intermediate_bindings::{
        codes_get_bytes, codes_get_double, codes_get_double_array, codes_get_long,
        codes_get_long_array, codes_get_native_type, codes_get_size, codes_get_string,
        NativeKeyType,
    },
    Key, KeyType, KeyedMessage,
};

impl KeyedMessage {
    /// Method to get a [`Key`] with provided name from the `KeyedMessage`, if it exists.
    /// 
    /// This function check the type of requested key and tries to read it as the native type.
    /// That flow adds performance overhead, but makes the function highly unlikely to fail.
    /// 
    /// This function will try to retrieve the key of native string type as string even
    /// when the nul byte is not positioned at the end of key value.
    /// 
    /// If retrieving the key value in native type fails this function will try to read
    /// the requested key as bytes.
    /// 
    /// # Example
    /// 
    /// ```
    ///  use eccodes::{ProductKind, CodesHandle, KeyType};
    ///  # use std::path::Path;
    ///  # use anyhow::Context;
    ///  use eccodes::FallibleStreamingIterator;
    ///  #
    ///  # fn main() -> anyhow::Result<()> {
    ///  let file_path = Path::new("./data/iceland.grib");
    ///  let product_kind = ProductKind::GRIB;
    ///  
    ///  let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
    ///  let message = handle.next()?.context("no message")?;
    ///  let message_short_name = message.read_key("shortName")?;
    ///  let expected_short_name = KeyType::Str("msl".to_string());
    ///  
    ///  assert_eq!(message_short_name.value, expected_short_name);
    ///  # Ok(())
    ///  # }
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns [`CodesNotFound`](crate::errors::CodesInternal::CodesNotFound)
    /// when a key of given name has not been found in the message.
    /// 
    /// Returns [`CodesError::MissingKey`] when a given key does not have a specified type.
    /// 
    /// Returns [`CodesError::Internal`] when one of internal ecCodes functions to read the key fails.
    /// 
    /// Returns [`CodesError::CstrUTF8`] and [`CodesError::NulChar`] when the string returned by ecCodes
    /// library cannot be parsed as valid UTF8 Rust string.
    /// 
    /// Returns [`CodesError::IncorrectKeySize`] when the size of given key is lower than 1. This indicates corrupted data file,
    /// bug in the crate or bug in the ecCodes library. If you encounter this error please check
    /// if your file is correct and report it on Github.
    pub fn read_key(&self, key_name: &str) -> Result<Key, CodesError> {
        let key_type;

        unsafe {
            key_type = codes_get_native_type(self.message_handle, key_name)?;
        }

        let key_value = match key_type {
            NativeKeyType::Long => {
                let key_size;
                unsafe { key_size = codes_get_size(self.message_handle, key_name)? }

                if key_size == 1 {
                    let value;
                    unsafe {
                        value = codes_get_long(self.message_handle, key_name);
                    }

                    match value {
                        Ok(val) => Ok(KeyType::Int(val)),
                        Err(err) => Err(err),
                    }
                } else if key_size >= 2 {
                    let value;
                    unsafe {
                        value = codes_get_long_array(self.message_handle, key_name);
                    }

                    match value {
                        Ok(val) => Ok(KeyType::IntArray(val)),
                        Err(err) => Err(err),
                    }
                } else {
                    return Err(CodesError::IncorrectKeySize)
                }
            }
            NativeKeyType::Double => {
                let key_size;
                unsafe { key_size = codes_get_size(self.message_handle, key_name)? }

                if key_size == 1 {
                    let value;
                    unsafe {
                        value = codes_get_double(self.message_handle, key_name);
                    }

                    match value {
                        Ok(val) => Ok(KeyType::Float(val)),
                        Err(err) => Err(err),
                    }
                } else if key_size >= 2 {
                    let value;
                    unsafe {
                        value = codes_get_double_array(self.message_handle, key_name);
                    }

                    match value {
                        Ok(val) => Ok(KeyType::FloatArray(val)),
                        Err(err) => Err(err),
                    }
                } else {
                    return Err(CodesError::IncorrectKeySize)
                }
            }
            NativeKeyType::Bytes => {
                let value;
                unsafe {
                    value = codes_get_bytes(self.message_handle, key_name);
                }

                match value {
                    Ok(val) => Ok(KeyType::Bytes(val)),
                    Err(err) => Err(err),
                }
            }
            NativeKeyType::Missing => return Err(CodesError::MissingKey),
            _ => {
                let value;
                unsafe {
                    value = codes_get_string(self.message_handle, key_name);
                }

                match value {
                    Ok(val) => Ok(KeyType::Str(val)),
                    Err(err) => Err(err),
                }
            }
        };

        if let Ok(value) = key_value {
            Ok(Key {
                name: key_name.to_owned(),
                value,
            })
        } else {
            let value;
            unsafe {
                value = codes_get_bytes(self.message_handle, key_name)?;
            }

            Ok(Key {
                name: key_name.to_owned(),
                value: KeyType::Bytes(value),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};

    use crate::codes_handle::{CodesHandle, ProductKind};
    use crate::{FallibleIterator, FallibleStreamingIterator, KeyType};
    use std::path::Path;

    #[test]
    fn key_reader() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;

        let current_message = handle.next()?.context("Message not some")?;

        let str_key = current_message.read_key("name")?;

        match str_key.value {
            KeyType::Str(_) => {}
            _ => panic!("Incorrect variant of string key"),
        }

        assert_eq!(str_key.name, "name");

        let double_key = current_message.read_key("jDirectionIncrementInDegrees")?;
        match double_key.value {
            KeyType::Float(_) => {}
            _ => panic!("Incorrect variant of double key"),
        }

        assert_eq!(double_key.name, "jDirectionIncrementInDegrees");

        let long_key = current_message.read_key("numberOfPointsAlongAParallel")?;

        match long_key.value {
            KeyType::Int(_) => {}
            _ => panic!("Incorrect variant of long key"),
        }

        assert_eq!(long_key.name, "numberOfPointsAlongAParallel");

        let double_arr_key = current_message.read_key("values")?;

        match double_arr_key.value {
            KeyType::FloatArray(_) => {}
            _ => panic!("Incorrect variant of double array key"),
        }

        assert_eq!(double_arr_key.name, "values");

        Ok(())
    }

    #[test]
    fn era5_keys() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.context("Message not some")?;
        let mut kiter = current_message.default_keys_iterator()?;

        while let Some(key) = kiter.next()? {
            assert!(!key.name.is_empty());
        }

        Ok(())
    }

    #[test]
    fn gfs_keys() -> Result<()> {
        let file_path = Path::new("./data/gfs.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.context("Message not some")?;
        let mut kiter = current_message.default_keys_iterator()?;

        while let Some(key) = kiter.next()? {
            assert!(!key.name.is_empty());
        }

        Ok(())
    }

    #[test]
    fn missing_key() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.context("Message not some")?;

        let missing_key = current_message.read_key("doesNotExist");

        assert!(missing_key.is_err());

        Ok(())
    }

    #[test]
    fn benchmark_keys() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;

        let msg = handle.next()?.context("Message not some")?;

        let _ = msg.read_key("dataDate")?;
        let _ = msg.read_key("jDirectionIncrementInDegrees")?;
        let _ = msg.read_key("values")?;
        let _ = msg.read_key("name")?;
        let _ = msg.read_key("section1Padding")?;
        let _ = msg.read_key("experimentVersionNumber")?;
        let _ = msg
            .read_key("zero")
            .unwrap_or_else(|_| msg.read_key("zeros").unwrap());

        Ok(())
    }
}
