use std::{fs::OpenOptions, io::Write, path::Path, slice};

use crate::{
    errors::CodesError,
    intermediate_bindings::{
        codes_get_message, codes_set_bytes, codes_set_double, codes_set_double_array,
        codes_set_long, codes_set_long_array, codes_set_string,
    },
    KeyedMessage,
};

use super::KeyWrite;

impl KeyWrite<i64> for KeyedMessage {
    fn write_key(&mut self, name: &str, value: i64) -> Result<(), CodesError> {
        unsafe { codes_set_long(self.message_handle, name, value) }
    }
}

impl KeyWrite<f64> for KeyedMessage {
    fn write_key(&mut self, name: &str, value: f64) -> Result<(), CodesError> {
        unsafe { codes_set_double(self.message_handle, name, value) }
    }
}

impl KeyWrite<&[i64]> for KeyedMessage {
    fn write_key(&mut self, name: &str, value: &[i64]) -> Result<(), CodesError> {
        unsafe { codes_set_long_array(self.message_handle, name, value) }
    }
}

impl KeyWrite<&[f64]> for KeyedMessage {
    fn write_key(&mut self, name: &str, value: &[f64]) -> Result<(), CodesError> {
        unsafe { codes_set_double_array(self.message_handle, name, value) }
    }
}

impl KeyWrite<&[u8]> for KeyedMessage {
    fn write_key(&mut self, name: &str, value: &[u8]) -> Result<(), CodesError> {
        unsafe { codes_set_bytes(self.message_handle, name, value) }
    }
}

impl KeyWrite<&str> for KeyedMessage {
    fn write_key(&mut self, name: &str, value: &str) -> Result<(), CodesError> {
        unsafe { codes_set_string(self.message_handle, name, value) }
    }
}

impl KeyedMessage {
    /// Function to write given `KeyedMessage` to a file at provided path.
    /// If file does not exists it will be created.
    /// If `append` is set to `true` file will be opened in append mode
    /// and no data will be overwritten (useful when writing mutiple messages to one file).
    ///
    /// # Example
    ///
    /// ```
    ///  use eccodes::{CodesHandle, Key, KeyOps, ProductKind};
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
    pub fn write_to_file(&self, file_path: &Path, append: bool) -> Result<(), CodesError> {
        let msg = unsafe { codes_get_message(self.message_handle)? };
        let buf = unsafe { slice::from_raw_parts(msg.0.cast::<u8>(), msg.1) };
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

    use crate::{
        codes_handle::{CodesHandle, ProductKind},
        DynamicKeyType, FallibleStreamingIterator, KeyWrite,
    };
    use std::{fs::remove_file, path::Path};

    #[test]
    fn write_message_ref() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;

        let current_message = handle.next()?.context("Message not some")?;
        let out_path = Path::new("./data/iceland_write.grib");
        current_message.write_to_file(out_path, false)?;

        remove_file(out_path)?;

        Ok(())
    }

    #[test]
    fn write_message_clone() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.context("Message not some")?.try_clone()?;

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
        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.context("Message not some")?;
        current_message.write_to_file(out_path, false)?;

        let file_path = Path::new("./data/iceland-levels.grib");
        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.context("Message not some")?;
        current_message.write_to_file(out_path, true)?;

        remove_file(out_path)?;

        Ok(())
    }

    #[test]
    fn write_key() -> Result<()> {
        let product_kind = ProductKind::GRIB;
        let file_path = Path::new("./data/iceland.grib");

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let mut current_message = handle.next()?.context("Message not some")?.try_clone()?;

        let old_key = current_message.read_key_dynamic("centre")?;

        current_message.write_key("centre", "cnmc")?;

        let read_key = current_message.read_key_dynamic("centre")?;

        assert_ne!(old_key, read_key);
        assert_eq!(read_key.name, "centre");
        assert_eq!(read_key.value, DynamicKeyType::Str("cnmc".into()));

        Ok(())
    }

    #[test]
    fn edit_keys_and_save() -> Result<()> {
        let product_kind = ProductKind::GRIB;
        let file_path = Path::new("./data/iceland.grib");

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let mut current_message = handle.next()?.context("Message not some")?.try_clone()?;

        let old_key = current_message.read_key_dynamic("centre")?;

        current_message.write_key("centre", "cnmc")?;

        current_message.write_to_file(Path::new("./data/iceland_edit.grib"), false)?;

        let file_path = Path::new("./data/iceland_edit.grib");

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.context("Message not some")?;

        let read_key = current_message.read_key_dynamic("centre")?;

        assert_ne!(old_key, read_key);
        assert_eq!(read_key.name, "centre");
        assert_eq!(read_key.value, DynamicKeyType::Str("cnmc".into()));

        remove_file(Path::new("./data/iceland_edit.grib"))?;

        Ok(())
    }
}
