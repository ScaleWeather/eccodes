use eccodes_sys::codes_keys_iterator;
use fallible_iterator::FallibleIterator;

use crate::{
    codes_handle::{Key, KeyedMessage},
    errors::CodesError,
    intermediate_bindings::{
        codes_keys_iterator_get_name, codes_keys_iterator_new, codes_keys_iterator_next,
    },
};

use super::KeysIteratorFlags;

///`FallibleIterator` implementation for `KeyedMessage` to access keyes inside message.
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
///# use eccodes::FallibleIterator;
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

impl KeyedMessage {
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
    ///# use eccodes::FallibleIterator;
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
                    None => String::new(),
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
}

#[cfg(test)]
mod tests {
    use crate::codes_handle::{CodesHandle, KeysIteratorFlags, ProductKind};
    use crate::FallibleIterator;
    use std::path::Path;

    #[test]
    fn keys_iterator_parameters() {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();
        let mut current_message = handle.next().unwrap().unwrap();

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
