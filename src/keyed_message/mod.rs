//! Definition of `KeyedMessage` and its associated functions
//! used for reading and writing data of given variable from GRIB file

mod read;
mod write;

use eccodes_sys::codes_handle;
use log::warn;
use std::ptr::null_mut;

use crate::{
    intermediate_bindings::{
        codes_get_native_type, codes_get_size, codes_handle_clone, codes_handle_delete,
        NativeKeyType,
    },
    CodesError,
};

/// Structure that provides access to the data contained in the GRIB file, which directly corresponds to the message in the GRIB file
///
/// **Usage examples are provided in documentation of each method.**
///
/// You can think about the message as a container of data corresponding to a single variable
/// at given date, time and level. In ecCodes the message is represented as a collection of unique
/// key-value pairs.
///
/// You can read a `Key` with static types using [`read_key()`](KeyRead::read_key()) or with [`DynamicKeyType`] using[`read_key_dynamic()`](KeyedMessage::read_key_dynamic())
/// To iterate over all key names use [`KeysIterator`](crate::KeysIterator). You can also modify the message using
/// [`write_key()`](KeyWrite::write_key()). This crate can successfully read all keys from ERA5 and GFS files.
///
/// If you are interested only in getting data values from the message you can use
/// [`to_ndarray()`](KeyedMessage::to_ndarray) from the [`message_ndarray`](crate::message_ndarray) module.
///
/// Some of the useful keys are: `validityDate`, `validityTime`, `level`, `typeOfLevel`, `shortName`, `units` and `values`.
///
/// Note that names, types and availability of some keys can vary between platforms and ecCodes versions. You should test
/// your code whenever changing the environment.
///
/// [`CodesNearest`](crate::CodesNearest) can be used to find nearest gridpoints for given coordinates in the `KeyedMessage`.
///
/// Most of `KeyedMessage` methods (except for writing) can be used directly with `&KeyedMessage`
/// returned by `CodesHandle` iterator, which provides the best performance.
/// When mutable access or longer liftime is needed the message can be cloned with [`try_clone`](KeyedMessage::try_clone)
/// Note that cloning comes with a performance and memory overhead.
/// You should take care that your system has enough memory before cloning.
///
/// Destructor for this structure does not panic, but some internal functions may rarely fail
/// leading to bugs. Errors encountered in desctructor the are logged with [`log`].
#[derive(Hash, Debug)]
pub struct KeyedMessage {
    pub(crate) message_handle: *mut codes_handle,
}

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

/// Provides GRIB key writing capabilites. Implemented by [`KeyedMessage`] for all possible key types.
pub trait KeyWrite<T> {
    /// Writes key with given name and value to [`KeyedMessage`] overwriting existing value, unless 
    /// the key is read-only. This function directly calls ecCodes ensuring only type and memory safety.
    /// 
    /// # Example
    ///
    /// ```
    ///  # use eccodes::{ProductKind, CodesHandle, KeyWrite};
    ///  # use std::path::Path;
    ///  # use anyhow::Context;
    ///  # use eccodes::FallibleStreamingIterator;
    ///  #
    ///  # fn main() -> anyhow::Result<()> {
    ///  # let file_path = Path::new("./data/iceland.grib");
    ///  # let product_kind = ProductKind::GRIB;
    ///  #
    ///  let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
    /// 
    /// // CodesHandle iterator returns immutable messages.
    /// // To edit a message it must be cloned.
    ///  let mut message = handle.next()?.context("no message")?.try_clone()?;
    ///  message.write_key("level", 1)?;
    ///  # Ok(())
    ///  # }
    /// ```
    /// 
    /// # Errors
    /// 
    /// This function will return [`CodesInternal`](crate::errors::CodesInternal) if ecCodes fails to write the key.
    fn write_key(&mut self, name: &str, value: T) -> Result<(), CodesError>;
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

impl KeyedMessage {
    /// Custom function to clone the `KeyedMessage`. This function comes with memory overhead.
    ///
    /// # Errors
    /// This function will return [`CodesInternal`](crate::errors::CodesInternal) if ecCodes fails to clone the message.
    pub fn try_clone(&self) -> Result<Self, CodesError> {
        let new_handle = unsafe { codes_handle_clone(self.message_handle)? };

        Ok(Self {
            message_handle: new_handle,
        })
    }

    fn get_key_size(&self, key_name: &str) -> Result<usize, CodesError> {
        unsafe { codes_get_size(self.message_handle, key_name) }
    }

    fn get_key_native_type(&self, key_name: &str) -> Result<NativeKeyType, CodesError> {
        unsafe { codes_get_native_type(self.message_handle, key_name) }
    }
}

#[doc(hidden)]
impl Drop for KeyedMessage {
    /// Executes the destructor for this type.
    /// This method calls destructor functions from ecCodes library.
    /// In some edge cases these functions can return non-zero code.
    /// In such case all pointers and file descriptors are safely deleted.
    /// However memory leaks can still occur.
    ///
    /// If any function called in the destructor returns an error warning will appear in log.
    /// If bugs occur during `CodesHandle` drop please enable log output and post issue on [Github](https://github.com/ScaleWeather/eccodes).
    ///
    /// Technical note: delete functions in ecCodes can only fail with [`CodesInternalError`](crate::errors::CodesInternal::CodesInternalError)
    /// when other functions corrupt the inner memory of pointer, in that case memory leak is possible.
    /// In case of corrupt pointer segmentation fault will occur.
    /// The pointers are cleared at the end of drop as they are not functional regardless of result of delete functions.
    fn drop(&mut self) {
        unsafe {
            codes_handle_delete(self.message_handle).unwrap_or_else(|error| {
                warn!("codes_handle_delete() returned an error: {:?}", &error);
            });
        }

        self.message_handle = null_mut();
    }
}

#[cfg(test)]
mod tests {
    use crate::codes_handle::{CodesHandle, ProductKind};
    use crate::FallibleStreamingIterator;
    use anyhow::{Context, Result};
    use std::path::Path;
    use testing_logger;

    #[test]
    fn check_docs_keys() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.context("Message not some")?;

        let _ = current_message.read_key_dynamic("validityDate")?;
        let _ = current_message.read_key_dynamic("validityTime")?;
        let _ = current_message.read_key_dynamic("level")?;
        let _ = current_message.read_key_dynamic("shortName")?;
        let _ = current_message.read_key_dynamic("units")?;
        let _ = current_message.read_key_dynamic("values")?;
        let _ = current_message.read_key_dynamic("typeOfLevel")?;

        Ok(())
    }

    #[test]
    fn message_clone_1() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.context("Message not some")?;
        let cloned_message = current_message.try_clone()?;

        assert_ne!(
            current_message.message_handle,
            cloned_message.message_handle
        );

        Ok(())
    }

    #[test]
    fn message_clone_2() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let msg = handle.next()?.context("Message not some")?.try_clone()?;
        let _ = handle.next()?;

        drop(handle);

        let _ = msg.read_key_dynamic("dataDate")?;
        let _ = msg.read_key_dynamic("jDirectionIncrementInDegrees")?;
        let _ = msg.read_key_dynamic("values")?;
        let _ = msg.read_key_dynamic("name")?;
        let _ = msg.read_key_dynamic("section1Padding")?;
        let _ = msg.read_key_dynamic("experimentVersionNumber")?;

        Ok(())
    }

    #[test]
    fn message_drop() -> Result<()> {
        testing_logger::setup();
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.context("Message not some")?.try_clone()?;

        let _kiter = current_message.default_keys_iterator()?;
        let _niter = current_message.codes_nearest()?;

        drop(handle);
        drop(_kiter);
        drop(_niter);
        drop(current_message);

        testing_logger::validate(|captured_logs| {
            assert_eq!(captured_logs.len(), 0);
        });

        Ok(())
    }
}
