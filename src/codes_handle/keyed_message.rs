use crate::{
    codes_handle::{KeyedMessage, Key},
    errors::CodesError,
    intermediate_bindings::{
        codes_get_double, codes_get_double_array, codes_get_long, codes_get_long_array,
        codes_get_native_type, codes_get_size, codes_get_string, NativeKeyType,
    },
};

impl KeyedMessage {
    ///Method to get a [`Key`] with provided name from the `KeyedMessage`.
    ///
    ///This function takes a key name and returns the key value as [`Key`]
    ///if requested key exists.
    ///
    ///## Example
    ///
    ///```
    ///# use eccodes::codes_handle::{ProductKind, CodesHandle, Key::Str};
    ///# use std::path::Path;
    ///#
    ///let file_path = Path::new("./data/iceland.grib");
    ///let product_kind = ProductKind::GRIB;
    ///
    ///let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();
    ///let message = handle.next().unwrap().unwrap();
    ///let message_short_name = message.read_key("shortName").unwrap();
    ///
    ///assert_eq!(message_short_name, Str(String::from("msl")));
    ///```
    ///
    ///## Errors
    ///
    ///Returns [`CodesInternal::CodesNotFound`](crate::errors::CodesInternal::CodesNotFound)
    ///wrapped in [`CodesError::Internal`] when a key of given name has not been found in the message.
    ///
    ///Returns [`CodesError::Internal`] when one of internal ecCodes functions to read the key fails.
    ///
    ///Returns [`CodesError::CstrUTF8`] and [`CodesError::NulChar`] when the string returned by ecCodes
    ///library cannot be parsed as valid UTF8 Rust string.
    ///
    ///## Panics
    ///
    ///Panics when the size of given key is lower than 1. This indicates corrupted data file,
    ///bug in the crate or bug in the ecCodes library. If you encounter this panic please check
    ///if your file is correct and report it on Github. 
    pub fn read_key(&self, key: &str) -> Result<Key, CodesError> {
        let key_type;

        unsafe {
            key_type = codes_get_native_type(self.file_handle, key)?;
        }

        match key_type {
            NativeKeyType::Long => {
                let key_size;
                unsafe { key_size = codes_get_size(self.file_handle, key)? }

                if key_size == 1 {
                    let key_value;
                    unsafe {
                        key_value = codes_get_long(self.file_handle, key)?;
                    }

                    Ok(Key::Int(key_value))
                } else if key_size > 2 {
                    let key_value;
                    unsafe {
                        key_value = codes_get_long_array(self.file_handle, key)?;
                    }

                    Ok(Key::IntArray(key_value))
                } else {
                    panic!("Incorrect key size!");
                }
            }
            NativeKeyType::Double => {
                let key_size;
                unsafe { key_size = codes_get_size(self.file_handle, key)? }

                if key_size == 1 {
                    let key_value;
                    unsafe {
                        key_value = codes_get_double(self.file_handle, key)?;
                    }

                    Ok(Key::Float(key_value))
                } else if key_size > 2 {
                    let key_value;
                    unsafe {
                        key_value = codes_get_double_array(self.file_handle, key)?;
                    }

                    Ok(Key::FloatArray(key_value))
                } else {
                    panic!("Incorrect key size!");
                }
            }
            NativeKeyType::Str => {
                let key_value;
                unsafe {
                    key_value = codes_get_string(self.file_handle, key)?;
                }

                Ok(Key::Str(key_value))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::codes_handle::{Key, CodesHandle, ProductKind};
    use std::path::Path;

    #[test]
    fn key_reader() {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();

        handle.next();

        let str_key = handle.current_message.read_key("name").unwrap();

        match str_key {
            Key::Str(_) => {}
            _ => panic!("Incorrect variant of string key"),
        }

        let double_key = handle
            .current_message
            .read_key("jDirectionIncrementInDegrees")
            .unwrap();

        match double_key {
            Key::Float(_) => {}
            _ => panic!("Incorrect variant of double key"),
        }

        let long_key = handle
            .current_message
            .read_key("numberOfPointsAlongAParallel")
            .unwrap();

        match long_key {
            Key::Int(_) => {}
            _ => panic!("Incorrect variant of long key"),
        }

        let double_arr_key = handle.current_message.read_key("values").unwrap();

        match double_arr_key {
            Key::FloatArray(_) => {}
            _ => panic!("Incorrect variant of double array key"),
        }
    }
}
