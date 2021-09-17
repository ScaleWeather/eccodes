use std::ptr::null_mut;
use log::warn;

use crate::{
    codes_handle::{Key, KeyedMessage, KeyType},
    errors::CodesError,
    intermediate_bindings::{
        codes_get_double, codes_get_double_array, codes_get_long, codes_get_long_array,
        codes_get_message_copy, codes_get_native_type, codes_get_size, codes_get_string,
        codes_handle_delete, codes_handle_new_from_message_copy, NativeKeyType,
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
    ///# use eccodes::codes_handle::{ProductKind, CodesHandle, KeyType::Str};
    ///# use std::path::Path;
    ///# use fallible_iterator::FallibleIterator;
    ///#
    ///let file_path = Path::new("./data/iceland.grib");
    ///let product_kind = ProductKind::GRIB;
    ///
    ///let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();
    ///let message = handle.next().unwrap().unwrap();
    ///let message_short_name = message.read_key("shortName").unwrap();
    ///
    ///assert_eq!(message_short_name.value, Str(String::from("msl")));
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
    pub fn read_key(&self, key_name: &str) -> Result<Key, CodesError> {
        let key_type;

        unsafe {
            key_type = codes_get_native_type(self.message_handle, key_name)?;
        }

        match key_type {
            NativeKeyType::Long => {
                let key_size;
                unsafe { key_size = codes_get_size(self.message_handle, key_name)? }

                if key_size == 1 {
                    let key_value;
                    unsafe {
                        key_value = codes_get_long(self.message_handle, key_name)?;
                    }

                    Ok(Key {
                        name: key_name.to_owned(),
                        value: KeyType::Int(key_value),
                    })
                } else if key_size > 2 {
                    let key_value;
                    unsafe {
                        key_value = codes_get_long_array(self.message_handle, key_name)?;
                    }

                    Ok(Key {
                        name: key_name.to_owned(),
                        value: KeyType::IntArray(key_value),
                    })
                } else {
                    panic!("Incorrect key size!");
                }
            }
            NativeKeyType::Double => {
                let key_size;
                unsafe { key_size = codes_get_size(self.message_handle, key_name)? }

                if key_size == 1 {
                    let key_value;
                    unsafe {
                        key_value = codes_get_double(self.message_handle, key_name)?;
                    }

                    Ok(Key {
                        name: key_name.to_owned(),
                        value: KeyType::Float(key_value),
                    })
                } else if key_size > 2 {
                    let key_value;
                    unsafe {
                        key_value = codes_get_double_array(self.message_handle, key_name)?;
                    }

                    Ok(Key {
                        name: key_name.to_owned(),
                        value: KeyType::FloatArray(key_value),
                    })
                } else {
                    panic!("Incorrect key size!");
                }
            }
            NativeKeyType::Str => {
                let key_value;
                unsafe {
                    key_value = codes_get_string(self.message_handle, key_name)?;
                }

                Ok(Key {
                    name: key_name.to_owned(),
                    value: KeyType::Str(key_value),
                })
            }
        }
    }

    pub fn set_iterator_parameters(&mut self) -> Result<(), CodesError> {
        Ok(())
    }
}

impl Clone for KeyedMessage {
    fn clone(&self) -> KeyedMessage {
        let new_handle;
        let new_buffer;

        unsafe {
            new_buffer = codes_get_message_copy(self.message_handle).expect("");
            new_handle = codes_handle_new_from_message_copy(&new_buffer);
        }

        KeyedMessage {
            message_handle: new_handle,
            message_buffer: new_buffer,
            iterator_flags: 0,
            iterator_namespace: "".to_owned(),
        }
    }
}

impl Drop for KeyedMessage {
    ///Executes the destructor for this type.
    ///This method calls `codes_handle_delete()` from ecCodes for graceful cleanup.
    ///
    ///Currently it is assumed that under normal circumstances this destructor never fails.
    ///However in some edge cases ecCodes can return non-zero code.
    ///In such case all pointers and file descriptors are safely deleted.
    ///However memory leaks can still occur.
    ///
    ///If any function called in the destructor returns an error warning will appear in log.
    ///If bugs occurs during `CodesHandle` drop please enable log output and post issue on [Github](https://github.com/ScaleWeather/eccodes).

    fn drop(&mut self) {
        //codes_handle_delete() can only fail with CodesInternalError when previous
        //functions corrupt the codes_handle, in that case memory leak is possible
        //moreover, if that happens the codes_handle is not functional so we clear it
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
    use crate::codes_handle::{KeyType, CodesHandle, ProductKind};
    use fallible_iterator::FallibleIterator;
    use std::path::Path;

    #[test]
    fn key_reader() {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();

        let current_message = handle.next().unwrap().unwrap();

        let str_key = current_message.read_key("name").unwrap();

        match str_key.value {
            KeyType::Str(_) => {}
            _ => panic!("Incorrect variant of string key"),
        }

        let double_key = current_message
            .read_key("jDirectionIncrementInDegrees")
            .unwrap();

        match double_key.value {
            KeyType::Float(_) => {}
            _ => panic!("Incorrect variant of double key"),
        }

        let long_key = current_message
            .read_key("numberOfPointsAlongAParallel")
            .unwrap();

        match long_key.value {
            KeyType::Int(_) => {}
            _ => panic!("Incorrect variant of long key"),
        }

        let double_arr_key = current_message.read_key("values").unwrap();

        match double_arr_key.value {
            KeyType::FloatArray(_) => {}
            _ => panic!("Incorrect variant of double array key"),
        }
    }
}
