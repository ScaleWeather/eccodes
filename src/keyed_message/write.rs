use std::{fs::OpenOptions, io::Write, path::Path, slice};

use crate::{
    codes_handle::{Key, KeyType}, errors::CodesError, intermediate_bindings::{
        codes_get_message, codes_set_bytes, codes_set_double, codes_set_double_array,
        codes_set_long, codes_set_long_array, codes_set_string,
    }, KeyedMessage
};

impl KeyedMessage {
    ///Function to write given `KeyedMessage` to a file at provided path.
    ///If file does not exists it will be created.
    ///If `append` is set to `true` file will be opened in append mode
    ///and no data will be overwritten (useful when writing mutiple messages to one file).
    ///
    ///## Example
    ///
    ///```
    ///# use eccodes::{
    ///#     codes_handle::{CodesHandle, Key, KeyType::Str, ProductKind::GRIB},
    ///#     errors::CodesError,
    ///# };
    ///# use crate::eccodes::FallibleIterator;
    ///# use std::path::Path;
    ///# use std::fs::remove_file;
    ///#
    ///# fn main() -> Result<(), CodesError> {
    ///let in_path = Path::new("./data/iceland-levels.grib");
    ///let out_path  = Path::new("./data/iceland-temperature-levels.grib");
    ///
    ///let handle = CodesHandle::new_from_file(in_path, GRIB)?;
    ///
    ///let mut t_levels =
    ///    handle.filter(|msg| Ok(msg.read_key("shortName")?.value == Str("t".to_string())));
    ///
    ///while let Some(msg) = t_levels.next()? {
    ///    msg.write_to_file(out_path, true)?;
    ///}
    ///# remove_file(out_path).unwrap();
    ///# Ok(())
    ///# }
    ///```
    ///
    ///## Errors
    ///
    ///Returns [`CodesError::FileHandlingInterrupted`] when the file cannot be opened,
    ///created or correctly written.
    ///
    ///Returns [`CodesInternal`](crate::errors::CodesInternal)
    ///when internal ecCodes function returns non-zero code.
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

    ///Function to set specified `Key` inside the `KeyedMessage`.
    ///This function automatically matches the `KeyType` and uses adequate
    ///internal ecCodes function to set the key.
    ///The message must be mutable to use this function.
    ///
    ///**User must provide the `Key` with correct type**, otherwise
    ///error will occur.
    ///Note that not all keys can be set, for example
    ///`"name"` and `shortName` are read-only. Trying to set such keys
    ///will result in error. Some keys can also be set using a non-native
    ///type (eg. `centre`), but [`read_key()`](KeyedMessage::read_key()) function will only read then
    ///in native type.
    ///
    ///Refer to [ecCodes library documentation](https://confluence.ecmwf.int/display/ECC/ecCodes+Home)
    ///for more details.
    ///
    ///## Example
    ///
    ///```
    ///# use eccodes::{
    ///#     codes_handle::{CodesHandle, Key, KeyType, ProductKind::GRIB},
    ///# };
    ///# use crate::eccodes::FallibleIterator;
    ///# use std::path::Path;
    ///#
    ///let file_path = Path::new("./data/iceland.grib");
    ///
    ///let mut handle = CodesHandle::new_from_file(file_path, GRIB).unwrap();
    ///let mut current_message = handle.next().unwrap().unwrap();
    ///
    ///let new_key = Key {
    ///    name: "centre".to_string(),
    ///    value: KeyType::Str("cnmc".to_string()),
    ///};
    ///
    ///current_message.write_key(new_key).unwrap();
    ///```
    ///
    ///## Errors
    ///
    ///This method will return [`CodesInternal`](crate::errors::CodesInternal)
    ///when internal ecCodes function returns non-zero code.
    pub fn write_key(&mut self, key: Key) -> Result<(), CodesError> {
        match key.value {
            KeyType::Float(val) => unsafe {
                codes_set_double(self.message_handle, &key.name, val)?;
            },
            KeyType::Int(val) => unsafe {
                codes_set_long(self.message_handle, &key.name, val)?;
            },
            KeyType::FloatArray(val) => unsafe {
                codes_set_double_array(self.message_handle, &key.name, &val)?;
            },
            KeyType::IntArray(val) => unsafe {
                codes_set_long_array(self.message_handle, &key.name, &val)?;
            },
            KeyType::Str(val) => unsafe {
                codes_set_string(self.message_handle, &key.name, &val)?;
            },
            KeyType::Bytes(val) => unsafe {
                codes_set_bytes(self.message_handle, &key.name, &val)?;
            },
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Ok, Result};

    use crate::{
        codes_handle::{
            CodesHandle, Key,
            KeyType::{self},
            ProductKind,
        },
        FallibleStreamingIterator,
    };
    use std::{fs::remove_file, path::Path};

    #[test]
    fn write_message_ref() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;

        let current_message = handle.next()?.unwrap();
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
        let current_message = handle.next()?.unwrap().clone();

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
        let current_message = handle.next()?.unwrap();
        current_message.write_to_file(out_path, false)?;

        let file_path = Path::new("./data/iceland-levels.grib");
        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.unwrap();
        current_message.write_to_file(out_path, true)?;

        remove_file(out_path)?;

        Ok(())
    }

    #[test]
    fn write_key() -> Result<()> {
        let product_kind = ProductKind::GRIB;
        let file_path = Path::new("./data/iceland.grib");

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let mut current_message = handle.next()?.unwrap().clone();

        let old_key = current_message.read_key("centre")?;

        let new_key = Key {
            name: "centre".to_string(),
            value: KeyType::Str("cnmc".to_string()),
        };

        current_message.write_key(new_key.clone())?;

        let read_key = current_message.read_key("centre")?;

        assert_eq!(new_key, read_key);
        assert_ne!(old_key, read_key);

        Ok(())
    }

    #[test]
    fn edit_keys_and_save() -> Result<()> {
        let product_kind = ProductKind::GRIB;
        let file_path = Path::new("./data/iceland.grib");

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let mut current_message = handle.next()?.unwrap().clone();

        let old_key = current_message.read_key("centre")?;

        let new_key = Key {
            name: "centre".to_string(),
            value: KeyType::Str("cnmc".to_string()),
        };

        current_message.write_key(new_key.clone())?;

        current_message.write_to_file(Path::new("./data/iceland_edit.grib"), false)?;

        let file_path = Path::new("./data/iceland_edit.grib");

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.unwrap();

        let read_key = current_message.read_key("centre")?;

        assert_eq!(new_key, read_key);
        assert_ne!(old_key, read_key);

        remove_file(Path::new("./data/iceland_edit.grib"))?;

        Ok(())
    }
}
