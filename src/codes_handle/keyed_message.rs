use eccodes_sys::{codes_keys_iterator, codes_nearest};
use fallible_iterator::FallibleIterator;
use log::warn;
use std::ptr::null_mut;

use crate::{
    codes_handle::{Key, KeyType, KeyedMessage},
    errors::CodesError,
    intermediate_bindings::{
        codes_get_bytes, codes_get_double, codes_get_double_array, codes_get_long,
        codes_get_long_array, codes_get_message_copy, codes_get_native_type, codes_get_size,
        codes_get_string, codes_grib_nearest_delete, codes_grib_nearest_find,
        codes_grib_nearest_new, codes_handle_delete, codes_handle_new_from_message_copy,
        codes_keys_iterator_delete, codes_keys_iterator_get_name, codes_keys_iterator_new,
        codes_keys_iterator_next, NativeKeyType,
    },
};

use super::{KeysIteratorFlags, NearestGridpoint};

impl KeyedMessage {
    ///Method to get a [`Key`] with provided name from the `KeyedMessage`.
    ///
    ///This function takes a key name and returns the key value as [`Key`]
    ///if requested key exists. Check the [`Key`] documentation for details
    ///of possible key types.
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
    ///assert_eq!(message_short_name.value, Str("msl".to_string()));
    ///```
    ///
    ///This function will try to retrieve the key of native string type as string even
    ///when the nul byte is not positioned at the end of key value.
    ///
    ///If retrieving the key value in native type fails this function will try to read
    ///the requested key as bytes.
    ///
    ///## Errors
    ///
    ///Returns [`CodesInternal::CodesNotFound`](crate::errors::CodesInternal::CodesNotFound)
    ///wrapped in [`CodesError::Internal`] when a key of given name has not been found in the message.
    ///
    ///Returns [`CodesError::MissingKey`] when a given key has a missing type.
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
                } else if key_size > 2 {
                    let value;
                    unsafe {
                        value = codes_get_long_array(self.message_handle, key_name);
                    }

                    match value {
                        Ok(val) => Ok(KeyType::IntArray(val)),
                        Err(err) => Err(err),
                    }
                } else {
                    panic!("Incorrect key size!");
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
                } else if key_size > 2 {
                    let value;
                    unsafe {
                        value = codes_get_double_array(self.message_handle, key_name);
                    }

                    match value {
                        Ok(val) => Ok(KeyType::FloatArray(val)),
                        Err(err) => Err(err),
                    }
                } else {
                    panic!("Incorrect key size!");
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

    ///Function that allows to set the flags and namespace for `FallibleIterator`.
    ///**Must be called before calling the iterator.** Changing the parameters
    ///after first call of `next()` will have no effect on the iterator.
    ///
    ///The flags are set by providing any combination of [`KeysIteratorFlags`]
    ///inside a vector. Check the documentation for the details of each flag meaning.
    ///
    ///Namespace is set simply as string, eg. `"ls"`, `"time"`, `"parameter"`, `"geography"`, `"statistics"`.
    ///Invalid namespace will result in empty iterator.
    ///
    ///Default parameters are [`AllKeys`](KeysIteratorFlags::AllKeys) flag and `""` namespace,
    ///which implies iteration over all keys available in the message.
    ///
    ///### Example
    ///
    ///```
    ///# use eccodes::codes_handle::{ProductKind, CodesHandle, KeyedMessage, KeysIteratorFlags};
    ///# use std::path::Path;
    ///# use eccodes::codes_handle::KeyType::Str;
    ///# use fallible_iterator::FallibleIterator;
    ///let file_path = Path::new("./data/iceland.grib");
    ///let product_kind = ProductKind::GRIB;
    ///
    ///let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();
    ///let mut current_message = handle.next().unwrap().unwrap();
    ///
    ///
    ///let flags = vec![
    ///    KeysIteratorFlags::AllKeys,
    ///    KeysIteratorFlags::SkipOptional,
    ///    KeysIteratorFlags::SkipReadOnly,
    ///    KeysIteratorFlags::SkipDuplicates,
    ///];
    ///
    ///let namespace = "geography".to_owned();
    ///
    ///current_message.set_iterator_parameters(flags, namespace);
    ///
    ///
    ///while let Some(key) = current_message.next().unwrap() {
    ///    println!("{:?}", key);
    ///}
    ///```
    pub fn set_iterator_parameters(&mut self, flags: Vec<KeysIteratorFlags>, namespace: String) {
        self.iterator_namespace = Some(namespace);

        let mut flags_sum = 0;

        for flag in flags {
            flags_sum += flag as u32;
        }

        self.iterator_flags = Some(flags_sum);
    }

    fn keys_iterator(&mut self) -> Result<*mut codes_keys_iterator, CodesError> {
        self.keys_iterator.map_or_else(
            || {
                let flags = self.iterator_flags.unwrap_or(0);

                let namespace = match self.iterator_namespace.clone() {
                    Some(n) => n,
                    None => "".to_owned(),
                };

                let itr;
                let next_item;
                unsafe {
                    itr = codes_keys_iterator_new(self.message_handle, flags, &namespace);
                    next_item = codes_keys_iterator_next(itr);
                }

                self.keys_iterator_next_item_exists = next_item;
                self.keys_iterator = Some(itr);

                Ok(itr)
            },
            Ok,
        )
    }

    fn nearest_handle(&mut self) -> Result<*mut codes_nearest, CodesError> {
        if let Some(nrst) = self.nearest_handle {
            Ok(nrst)
        } else {
            let nrst;

            unsafe {
                nrst = codes_grib_nearest_new(self.message_handle)?;
            }

            self.nearest_handle = Some(nrst);

            Ok(nrst)
        }
    }

    ///Function to get four [`NearestGridpoint`]s of a point represented by requested coordinates.
    ///
    ///The inputs are latitude and longitude of requested point in respectively degrees north and
    ///degreed east.
    ///
    ///In the output gridpoints, the value field  refers to parameter held by the `KeyedMessage`
    ///for which the function is called in adequate units,
    ///coordinates are in degrees north/east,
    ///and distance field represents the distance between requested point and output point in kilometers.
    ///
    ///### Example
    ///
    ///```
    ///# use eccodes::codes_handle::{ProductKind, CodesHandle, KeyedMessage, KeysIteratorFlags};
    ///# use std::path::Path;
    ///# use eccodes::codes_handle::KeyType::Str;
    ///# use fallible_iterator::FallibleIterator;
    ///let file_path = Path::new("./data/iceland.grib");
    ///let product_kind = ProductKind::GRIB;
    ///
    ///let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();
    ///let mut msg = handle.next().unwrap().unwrap();
    ///
    ///
    ///let out = msg.find_nearest(64.13, -21.89).unwrap();
    ///```
    ///
    ///### Errors
    ///
    ///This function returns [`CodesInternal`](crate::errors::CodesInternal) when
    ///one of ecCodes function returns the non-zero code.
    pub fn find_nearest(
        &mut self,
        lat: f64,
        lon: f64,
    ) -> Result<[NearestGridpoint; 4], CodesError> {
        let nrst = self.nearest_handle()?;
        let output_points;

        unsafe {
            output_points = codes_grib_nearest_find(self.message_handle, nrst, lat, lon)?;
        }

        Ok(output_points)
    }
}

impl Clone for KeyedMessage {
    ///Custom function to clone the `KeyedMessage`. This function comes with memory overhead.
    ///During clone iterator flags and namespace are not copied, and the iterator is reset.
    fn clone(&self) -> KeyedMessage {
        let new_handle;
        let new_buffer;

        unsafe {
            new_buffer = codes_get_message_copy(self.message_handle).expect(
                "Getting message clone failed.
            Please report this panic on Github",
            );
            new_handle = codes_handle_new_from_message_copy(&new_buffer);
        }

        KeyedMessage {
            message_handle: new_handle,
            message_buffer: new_buffer,
            iterator_flags: None,
            iterator_namespace: None,
            keys_iterator: None,
            keys_iterator_next_item_exists: false,
            nearest_handle: None,
        }
    }
}

///`FallibleIterator` implementation for KeyedMessage to access keyes inside message.
///Mainly useful to discover what keys are present inside the message.
///
///This function internally calls [`read_key()`](KeyedMessage::read_key()) function
///so it is probably more efficient to call that function directly only for keys you
///are interested in.
///
///[`FallibleIterator`](fallible_iterator::FallibleIterator) is used instead of classic `Iterator`
///because internal ecCodes functions can return internal error in some edge-cases.
///The usage of `FallibleIterator` is sligthly different than usage of `Iterator`,
///check its documentation for more details.
///
///## Example
///
///```
///# use eccodes::codes_handle::{ProductKind, CodesHandle, KeyedMessage, KeysIteratorFlags};
///# use std::path::Path;
///# use eccodes::codes_handle::KeyType::Str;
///# use fallible_iterator::FallibleIterator;
///let file_path = Path::new("./data/iceland.grib");
///let product_kind = ProductKind::GRIB;
///
///let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();
///let mut current_message = handle.next().unwrap().unwrap();
///
///while let Some(key) = current_message.next().unwrap() {
///    println!("{:?}", key);
///}
///```
///
///## Errors
///The `next()` method will return [`CodesInternal`](crate::errors::CodesInternal)
///when internal ecCodes function returns non-zero code.
impl FallibleIterator for KeyedMessage {
    type Item = Key;
    type Error = CodesError;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        let itr = self.keys_iterator()?;

        if self.keys_iterator_next_item_exists {
            let key_name;
            let next_item_exists;

            unsafe {
                key_name = codes_keys_iterator_get_name(itr)?;
                next_item_exists = codes_keys_iterator_next(itr);
            }

            let key = KeyedMessage::read_key(self, &key_name)?;

            self.keys_iterator_next_item_exists = next_item_exists;

            Ok(Some(key))
        } else {
            Ok(None)
        }
    }
}

impl Drop for KeyedMessage {
    ///Executes the destructor for this type.
    ///This method calls `codes_handle_delete()`, `codes_keys_iterator_delete()`
    ///`codes_grib_nearest_delete()` from ecCodes for graceful cleanup.
    ///However in some edge cases ecCodes can return non-zero code.
    ///In such case all pointers and file descriptors are safely deleted.
    ///However memory leaks can still occur.
    ///
    ///If any function called in the destructor returns an error warning will appear in log.
    ///If bugs occurs during `CodesHandle` drop please enable log output and post issue on [Github](https://github.com/ScaleWeather/eccodes).
    ///
    ///Technical note: delete functions in ecCodes can only fail with [`CodesInternalError`](crate::errors::CodesInternal::CodesInternalError)
    ///when other functions corrupt the inner memory of pointer, in that case memory leak is possible.
    ///In case of corrupt pointer segmentation fault will occur.
    ///The pointers are cleared at the end of drop as they ar not not functional despite the result of delete functions.
    fn drop(&mut self) {
        if let Some(nrst) = self.nearest_handle {
            unsafe {
                codes_grib_nearest_delete(nrst).unwrap_or_else(|error| {
                    warn!(
                        "codes_grib_nearest_delete() returned an error: {:?}",
                        &error
                    );
                });
            }
        }

        if let Some(kiter) = self.keys_iterator {
            unsafe {
                codes_keys_iterator_delete(kiter).unwrap_or_else(|error| {
                    warn!(
                        "codes_keys_iterator_delete() returned an error: {:?}",
                        &error
                    );
                });
            }
        }

        self.keys_iterator = Some(null_mut());

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
    use crate::codes_handle::{CodesHandle, KeyType, KeysIteratorFlags, ProductKind};
    use fallible_iterator::FallibleIterator;
    use std::path::Path;
    use testing_logger;

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

        assert_eq!(str_key.name, "name");

        let double_key = current_message
            .read_key("jDirectionIncrementInDegrees")
            .unwrap();

        match double_key.value {
            KeyType::Float(_) => {}
            _ => panic!("Incorrect variant of double key"),
        }

        assert_eq!(double_key.name, "jDirectionIncrementInDegrees");

        let long_key = current_message
            .read_key("numberOfPointsAlongAParallel")
            .unwrap();

        match long_key.value {
            KeyType::Int(_) => {}
            _ => panic!("Incorrect variant of long key"),
        }

        assert_eq!(long_key.name, "numberOfPointsAlongAParallel");

        let double_arr_key = current_message.read_key("values").unwrap();

        match double_arr_key.value {
            KeyType::FloatArray(_) => {}
            _ => panic!("Incorrect variant of double array key"),
        }

        assert_eq!(double_arr_key.name, "values");
    }

    #[test]
    fn key_clone() {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();
        let current_message = handle.next().unwrap().unwrap();
        let cloned_message = current_message.clone();

        assert_ne!(
            current_message.message_handle,
            cloned_message.message_handle
        );
        assert!(!cloned_message.message_buffer.is_empty());
        assert!(cloned_message.iterator_flags.is_none());
        assert!(cloned_message.iterator_namespace.is_none());
        assert!(cloned_message.keys_iterator.is_none());
        assert!(!cloned_message.keys_iterator_next_item_exists);
    }

    #[test]
    fn keys_iterator() {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();
        let mut current_message = handle.next().unwrap().unwrap();

        for i in 0..=300 {
            let key = current_message.next();
            println!("{}: {:?}", i, key);
        }
    }

    #[test]
    fn message_drop() {
        testing_logger::setup();
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();
        let mut current_message = handle.next().unwrap().unwrap();

        let _key = current_message.next().unwrap().unwrap();

        drop(current_message);

        testing_logger::validate(|captured_logs| {
            assert_eq!(captured_logs.len(), 0);
        });
    }

    #[test]
    fn keys_iterator_parameters() {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();
        let mut current_message = handle.next().unwrap().unwrap();

        assert!(current_message.message_buffer.is_empty());
        assert!(current_message.iterator_flags.is_none());
        assert!(current_message.iterator_namespace.is_none());
        assert!(current_message.keys_iterator.is_none());
        assert!(!current_message.keys_iterator_next_item_exists);

        let flags = vec![
            KeysIteratorFlags::AllKeys,        //0
            KeysIteratorFlags::SkipOptional,   //2
            KeysIteratorFlags::SkipReadOnly,   //1
            KeysIteratorFlags::SkipDuplicates, //32
        ];

        let namespace = "geography".to_owned();

        current_message.set_iterator_parameters(flags, namespace);

        assert_eq!(current_message.iterator_flags, Some(35));
        assert_eq!(
            current_message.iterator_namespace,
            Some("geography".to_owned())
        );

        while let Some(key) = current_message.next().unwrap() {
            assert!(!key.name.is_empty());
        }
    }

    #[test]
    fn find_nearest() {
        let file_path1 = Path::new("./data/iceland.grib");
        let file_path2 = Path::new("./data/iceland-surface.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle1 = CodesHandle::new_from_file(file_path1, product_kind).unwrap();
        let mut msg1 = handle1.next().unwrap().unwrap();

        let mut handle2 = CodesHandle::new_from_file(file_path2, product_kind).unwrap();
        let mut msg2 = handle2.next().unwrap().unwrap();

        let out1 = msg1.find_nearest(64.13, -21.89).unwrap();
        let out2 = msg2.find_nearest(64.13, -21.89).unwrap();

        assert!(out1[0].value > 10000.0);
        assert!(out2[3].index == 551);
        assert!(out1[1].lat == 64.0);
        assert!(out2[2].lon == -21.75);
        assert!(out1[0].distance > 15.0);
    }

    #[test]
    fn missing_key() {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();
        let current_message = handle.next().unwrap().unwrap();

        let missing_key = current_message.read_key("doesNotExist");

        assert!(missing_key.is_err());
    }

    #[test]
    fn invalid_namespace() {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();
        let mut current_message = handle.next().unwrap().unwrap();

        let flags = vec![
            KeysIteratorFlags::AllKeys, //0
        ];

        let namespace = "blabla".to_owned();

        current_message.set_iterator_parameters(flags, namespace);

        while let Some(key) = current_message.next().unwrap() {
            assert!(!key.name.is_empty());
        }
    }
}
