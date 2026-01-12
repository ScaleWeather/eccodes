use std::{fmt::Debug, fs::OpenOptions, io::Write, path::Path, slice};

use crate::{
    codes_message::{BufMessage, CodesMessage},
    errors::CodesError,
    intermediate_bindings::{
        codes_get_message, codes_set_bytes, codes_set_double, codes_set_double_array,
        codes_set_long, codes_set_long_array, codes_set_string,
    },
};

/// Provides GRIB key writing capabilites. Implemented by [`KeyedMessage`] for all possible key types.
pub trait KeyWrite<T> {
    /// Unchecked doesn't mean it's unsafe - just that there are no checks on Rust side in comparison to
    /// `read_key` which has such checks.
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
    fn write_key_unchecked(&mut self, name: &str, value: T) -> Result<&mut Self, CodesError>;
}

macro_rules! impl_key_write {
    ($ec_func:ident, $gen_type:ty) => {
        impl KeyWrite<$gen_type> for BufMessage {
            fn write_key_unchecked(
                &mut self,
                name: &str,
                value: $gen_type,
            ) -> Result<&mut Self, CodesError> {
                unsafe {
                    $ec_func(self.message_handle, name, value)?;
                }
                Ok(self)
            }
        }
    };
}

impl_key_write!(codes_set_long, i64);
impl_key_write!(codes_set_double, f64);
impl_key_write!(codes_set_long_array, &[i64]);
impl_key_write!(codes_set_double_array, &[f64]);
impl_key_write!(codes_set_bytes, &[u8]);
impl_key_write!(codes_set_string, &str);

impl<PA: Debug> CodesMessage<PA> {
    /// Function to write given `KeyedMessage` to a file at provided path.
    /// If file does not exists it will be created.
    /// If `append` is set to `true` file will be opened in append mode
    /// and no data will be overwritten (useful when writing mutiple messages to one file).
    ///
    /// # Example
    ///
    /// ```
    ///  use eccodes::{CodesHandle, KeyRead, ProductKind};
    ///  # use eccodes::errors::CodesError;
    ///  use eccodes::FallibleStreamingIterator;
    ///  # use std::path::Path;
    ///  # use std::fs::remove_file;
    ///  #
    ///  # fn main() -> anyhow::Result<(), CodesError> {
    ///  let in_path = Path::new("./data/iceland-levels.grib");
    ///  let out_path  = Path::new("./data/iceland-800hPa.grib");
    ///
    ///  let mut handle = CodesHandle::new_from_file(in_path, ProductKind::GRIB)?;
    ///
    ///  while let Some(msg) = handle.next()? {
    ///      let level: i64 = msg.read_key("level")?;
    ///      if level == 800 {
    ///          msg.write_to_file(out_path, true)?;
    ///      }
    ///  }
    ///  # remove_file(out_path)?;
    ///  # Ok(())
    ///  # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`CodesError::FileHandlingInterrupted`] when the file cannot be opened,
    /// created or correctly written.
    ///
    /// Returns [`CodesInternal`](crate::errors::CodesInternal)
    /// when internal ecCodes function returns non-zero code.
    pub fn write_to_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        append: bool,
    ) -> Result<(), CodesError> {
        let msg = unsafe { codes_get_message(self.message_handle)? };
        let buf = unsafe { slice::from_raw_parts(msg.0.cast::<_>(), msg.1) };
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(append)
            .open(file_path)?;

        file.write_all(buf)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};
    use fallible_iterator::FallibleIterator;

    use crate::{
        KeyRead,
        codes_file::{CodesFile, ProductKind},
        codes_message::{DynamicKeyType, KeyWrite},
    };
    use std::{fs::remove_file, path::Path};

    #[test]
    fn write_message_ref() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;

        let current_message = handle
            .ref_message_iter()
            .next()?
            .context("Message not some")?;
        let out_path = Path::new("./data/iceland_write.grib");
        current_message.write_to_file(out_path, false)?;

        remove_file(out_path)?;

        Ok(())
    }

    #[test]
    fn write_message_clone() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;
        let current_message = handle
            .ref_message_iter()
            .next()?
            .context("Message not some")?
            .try_clone()?;

        drop(handle);

        let out_path = Path::new("./data/iceland_write_clone.grib");
        current_message.write_to_file(out_path, false)?;

        remove_file(out_path)?;

        Ok(())
    }

    #[test]
    fn append_message() -> Result<()> {
        let product_kind = ProductKind::GRIB;
        let out_path = Path::new("./data/iceland_append.grib");

        let file_path = Path::new("./data/iceland-surface.grib");
        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;
        let current_message = handle
            .ref_message_iter()
            .next()?
            .context("Message not some")?;
        current_message.write_to_file(out_path, false)?;

        let file_path = Path::new("./data/iceland-levels.grib");
        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;
        let current_message = handle
            .ref_message_iter()
            .next()?
            .context("Message not some")?;
        current_message.write_to_file(out_path, true)?;

        remove_file(out_path)?;

        Ok(())
    }

    #[test]
    fn write_key() -> Result<()> {
        let product_kind = ProductKind::GRIB;
        let file_path = Path::new("./data/iceland.grib");

        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;
        let current_message = handle
            .ref_message_iter()
            .next()?
            .context("Message not some")?;

        let old_key = current_message.read_key_dynamic("centre")?;

        let mut cloned_message = current_message.try_clone()?;
        cloned_message.write_key_unchecked("centre", "cnmc")?;

        let read_key = cloned_message.read_key_dynamic("centre")?;

        assert_ne!(old_key, read_key);
        assert_eq!(read_key, DynamicKeyType::Str("cnmc".into()));

        Ok(())
    }

    #[test]
    fn write_key_types() -> Result<()> {
        let product_kind = ProductKind::GRIB;
        let file_path = Path::new("./data/iceland.grib");

        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;
        let mut message = handle
            .ref_message_iter()
            .next()?
            .context("Message not some")?
            .try_clone()?;

        let mut values_array: Vec<f64> = message.read_key("values")?;
        values_array[0] = 0.0;

        message.write_key_unchecked("centre", "cnmc")?; // str
        message.write_key_unchecked("day", 3)?; // int
        message.write_key_unchecked("latitudeOfFirstGridPointInDegrees", 7.0)?; // float
        message.write_key_unchecked("values", values_array.as_slice())?; //float array

        // int array and bytes cannot be tested because written key must exist and must be writable
        Ok(())
    }

    #[test]
    fn edit_keys_and_save() -> Result<()> {
        let product_kind = ProductKind::GRIB;
        let file_path = Path::new("./data/iceland.grib");

        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;
        let current_message = handle
            .ref_message_iter()
            .next()?
            .context("Message not some")?;

        let old_key = current_message.read_key_dynamic("centre")?;

        let mut cloned_message = current_message.try_clone()?;
        cloned_message.write_key_unchecked("centre", "cnmc")?;

        cloned_message.write_to_file(Path::new("./data/iceland_edit.grib"), false)?;

        let file_path = Path::new("./data/iceland_edit.grib");

        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;
        let current_message = handle
            .ref_message_iter()
            .next()?
            .context("Message not some")?;

        let read_key = current_message.read_key_dynamic("centre")?;

        assert_ne!(old_key, read_key);
        assert_eq!(read_key, DynamicKeyType::Str("cnmc".into()));

        remove_file(Path::new("./data/iceland_edit.grib"))?;

        Ok(())
    }
}
