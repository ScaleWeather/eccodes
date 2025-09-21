use std::cmp::Ordering;

use crate::{
    KeyedMessage,
    codes_handle::ThreadSafeHandle,
    errors::CodesError,
    intermediate_bindings::{
        NativeKeyType, codes_get_bytes, codes_get_double, codes_get_double_array, codes_get_long,
        codes_get_long_array, codes_get_native_type, codes_get_size, codes_get_string,
    },
    keyed_message::AtomicMessage,
};

/// Provides GRIB key reading capabilites. Implemented by [`KeyedMessage`] for all possible key types.
pub trait KeyRead<T> {
    /// Tries to read a key of given name from [`KeyedMessage`]. This function checks if key native type
    /// matches the requested type (ie. you cannot read integer as string, or array as a number).
    ///
    /// # Example
    ///
    /// ```
    ///  # use eccodes::{ProductKind, CodesHandle, KeyRead};
    ///  # use std::path::Path;
    ///  # use anyhow::Context;
    ///  # use eccodes::FallibleStreamingIterator;
    ///  #
    ///  # fn main() -> anyhow::Result<()> {
    ///  # let file_path = Path::new("./data/iceland.grib");
    ///  # let product_kind = ProductKind::GRIB;
    ///  #
    ///  let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
    ///  let message = handle.next()?.context("no message")?;
    ///  let short_name: String = message.read_key("shortName")?;
    ///  
    ///  assert_eq!(short_name, "msl");
    ///  # Ok(())
    ///  # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`WrongRequestedKeySize`](CodesError::WrongRequestedKeyType) when trying to read key in non-native type (use [`unchecked`](KeyRead::read_key_unchecked) instead).
    ///
    /// Returns [`WrongRequestedKeySize`](CodesError::WrongRequestedKeySize) when trying to read array as integer.
    ///
    /// Returns [`IncorrectKeySize`](CodesError::IncorrectKeySize) when key size is 0. This can indicate corrupted data.
    ///
    /// This function will return [`CodesInternal`](crate::errors::CodesInternal) if ecCodes fails to read the key.
    fn read_key(&self, name: &str) -> Result<T, CodesError>;

    /// Skips all the checks provided by [`read_key`](KeyRead::read_key) and directly calls ecCodes, ensuring only memory and type safety.
    ///
    /// This function has better perfomance than [`read_key`](KeyRead::read_key) but all error handling and (possible)
    /// type conversions are performed directly by ecCodes.
    ///
    /// This function is also useful for (not usually used) keys that return incorrect native type.
    ///
    /// # Example
    ///
    /// ```
    ///  # use eccodes::{ProductKind, CodesHandle, KeyRead};
    ///  # use std::path::Path;
    ///  # use anyhow::Context;
    ///  # use eccodes::FallibleStreamingIterator;
    ///  #
    ///  # fn main() -> anyhow::Result<()> {
    ///  # let file_path = Path::new("./data/iceland.grib");
    ///  # let product_kind = ProductKind::GRIB;
    ///  #
    ///  let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
    ///  let message = handle.next()?.context("no message")?;
    ///  let short_name: String = message.read_key_unchecked("shortName")?;
    ///  
    ///  assert_eq!(short_name, "msl");
    ///  # Ok(())
    ///  # }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return [`CodesInternal`](crate::errors::CodesInternal) if ecCodes fails to read the key.
    fn read_key_unchecked(&self, name: &str) -> Result<T, CodesError>;
}

#[doc(hidden)]
pub trait KeyReadHelpers {
    fn get_key_size(&mut self, key_name: &str) -> Result<usize, CodesError>;
    fn get_key_native_type(&mut self, key_name: &str) -> Result<NativeKeyType, CodesError>;
}

impl KeyReadHelpers for KeyedMessage<'_> {
    fn get_key_size(&mut self, key_name: &str) -> Result<usize, CodesError> {
        unsafe { codes_get_size(self.message_handle, key_name) }
    }

    fn get_key_native_type(&mut self, key_name: &str) -> Result<NativeKeyType, CodesError> {
        unsafe { codes_get_native_type(self.message_handle, key_name) }
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

macro_rules! impl_key_read {
    ($key_sizing:ident, $ec_func:ident, $key_variant:path, $gen_type:ty) => {
        impl<S: ThreadSafeHandle> AtomicKeyRead<$gen_type> for AtomicMessage<S> {
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

/// Enum of types GRIB key can have.
///
/// Messages inside GRIB files can contain keys of arbitrary types, which are known only at runtime (after being checked).
/// ecCodes can return several different types of key, which are represented by this enum
/// and each variant contains the respective data type.
#[derive(Clone, Debug, PartialEq)]
pub enum DynamicKeyType {
    #[allow(missing_docs)]
    Float(f64),
    #[allow(missing_docs)]
    Int(i64),
    #[allow(missing_docs)]
    FloatArray(Vec<f64>),
    #[allow(missing_docs)]
    IntArray(Vec<i64>),
    #[allow(missing_docs)]
    Str(String),
    #[allow(missing_docs)]
    Bytes(Vec<u8>),
}

impl KeyedMessage<'_> {
    /// Method to get a value of given key with [`DynamicKeyType`] from the `KeyedMessage`, if it exists.
    ///
    /// In most cases you should use [`read_key()`](KeyRead::read_key) due to more predictive behaviour
    /// and simpler interface.
    ///
    /// This function exists for backwards compatibility and user convienience.
    ///
    /// This function checks the type of requested key and tries to read it as the native type.
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
    ///  use eccodes::{ProductKind, CodesHandle, DynamicKeyType};
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
    ///  let message_short_name = message.read_key_dynamic("shortName")?;
    ///  let expected_short_name = DynamicKeyType::Str("msl".to_string());
    ///  
    ///  assert_eq!(message_short_name, expected_short_name);
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
    pub fn read_key_dynamic(&self, key_name: &str) -> Result<DynamicKeyType, CodesError> {
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
                        Ok(val) => Ok(DynamicKeyType::Int(val)),
                        Err(err) => Err(err),
                    }
                } else if key_size >= 2 {
                    let value;
                    unsafe {
                        value = codes_get_long_array(self.message_handle, key_name);
                    }

                    match value {
                        Ok(val) => Ok(DynamicKeyType::IntArray(val)),
                        Err(err) => Err(err),
                    }
                } else {
                    return Err(CodesError::IncorrectKeySize);
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
                        Ok(val) => Ok(DynamicKeyType::Float(val)),
                        Err(err) => Err(err),
                    }
                } else if key_size >= 2 {
                    let value;
                    unsafe {
                        value = codes_get_double_array(self.message_handle, key_name);
                    }

                    match value {
                        Ok(val) => Ok(DynamicKeyType::FloatArray(val)),
                        Err(err) => Err(err),
                    }
                } else {
                    return Err(CodesError::IncorrectKeySize);
                }
            }
            NativeKeyType::Bytes => {
                let value;
                unsafe {
                    value = codes_get_bytes(self.message_handle, key_name);
                }

                match value {
                    Ok(val) => Ok(DynamicKeyType::Bytes(val)),
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
                    Ok(val) => Ok(DynamicKeyType::Str(val)),
                    Err(err) => Err(err),
                }
            }
        };

        if let Ok(value) = key_value {
            Ok(value)
        } else {
            let value;
            unsafe {
                value = codes_get_bytes(self.message_handle, key_name)?;
            }

            Ok(DynamicKeyType::Bytes(value))
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};

    use crate::codes_handle::{CodesHandle, ProductKind};
    use crate::{FallibleIterator, keyed_message::DynamicKeyType};
    use std::path::Path;

    #[test]
    fn key_reader() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;

        let current_message = handle
            .message_generator()
            .next()?
            .context("Message not some")?;

        let str_key = current_message.read_key_dynamic("name")?;

        match str_key {
            DynamicKeyType::Str(_) => {}
            _ => panic!("Incorrect variant of string key"),
        }

        let double_key = current_message.read_key_dynamic("jDirectionIncrementInDegrees")?;
        match double_key {
            DynamicKeyType::Float(_) => {}
            _ => panic!("Incorrect variant of double key"),
        }

        let long_key = current_message.read_key_dynamic("numberOfPointsAlongAParallel")?;

        match long_key {
            DynamicKeyType::Int(_) => {}
            _ => panic!("Incorrect variant of long key"),
        }

        let double_arr_key = current_message.read_key_dynamic("values")?;

        match double_arr_key {
            DynamicKeyType::FloatArray(_) => {}
            _ => panic!("Incorrect variant of double array key"),
        }

        Ok(())
    }

    #[test]
    fn era5_keys_dynamic() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle
            .message_generator()
            .next()?
            .context("Message not some")?;
        let mut kiter = current_message.default_keys_iterator()?;

        while let Some(key_name) = kiter.next()? {
            assert!(!key_name.is_empty());
            assert!(current_message.read_key_dynamic(&key_name).is_ok())
        }

        Ok(())
    }

    #[test]
    fn gfs_keys_dynamic() -> Result<()> {
        let file_path = Path::new("./data/gfs.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle
            .message_generator()
            .next()?
            .context("Message not some")?;
        let mut kiter = current_message.default_keys_iterator()?;

        while let Some(key_name) = kiter.next()? {
            assert!(!key_name.is_empty());
            assert!(current_message.read_key_dynamic(&key_name).is_ok())
        }

        Ok(())
    }

    #[test]
    fn missing_key() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle
            .message_generator()
            .next()?
            .context("Message not some")?;

        let missing_key = current_message.read_key_dynamic("doesNotExist");

        assert!(missing_key.is_err());

        Ok(())
    }

    #[test]
    fn benchmark_keys() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;

        let msg = handle
            .message_generator()
            .next()?
            .context("Message not some")?;

        let _ = msg.read_key_dynamic("dataDate")?;
        let _ = msg.read_key_dynamic("jDirectionIncrementInDegrees")?;
        let _ = msg.read_key_dynamic("values")?;
        let _ = msg.read_key_dynamic("name")?;
        let _ = msg.read_key_dynamic("section1Padding")?;
        let _ = msg.read_key_dynamic("experimentVersionNumber")?;
        let _ = msg
            .read_key_dynamic("zero")
            .unwrap_or_else(|_| msg.read_key_dynamic("zeros").unwrap());

        Ok(())
    }
}
