use eccodes_sys::codes_handle;
use fallible_iterator::FallibleIterator;

use crate::{
    codes_handle::{CodesHandle, KeyedMessage},
    errors::CodesError,
    intermediate_bindings::{
        codes_get_message, codes_handle_new_from_file, codes_handle_new_from_message,
    },
};

///`FallibleIterator` implementation for `CodesHandle` to access GRIB messages inside file.
///
///To access GRIB messages the ecCodes library uses a method similar to a C-style iterator.
///It digests the `FILE *` multiple times each time returning the `codes_handle` raw pointer
///to a message inside the file. This method would be unsafe to expose directly.
///Therefore this crate utilizes the `Iterator` to provide the access to GRIB messages in
///a safe and convienient way.
///
///[`FallibleIterator`](fallible_iterator::FallibleIterator) is used instead of classic `Iterator`
///because internal ecCodes functions can return error codes when the GRIB file
///is corrupted and for some other reasons. The usage of `FallibleIterator` is sligthly different
///than usage of `Iterator`, check its documentation for more details.
///
///Using the `FallibleIterator` is the only way to read `KeyedMessage`s from the file.
///Its basic usage is simply with while let statement (similar to for loop for classic `Iterator`):
///
///```
///# use eccodes::codes_handle::{ProductKind, CodesHandle, KeyType};
///# use std::path::Path;
///# use fallible_iterator::FallibleIterator;
///#
///let file_path = Path::new("./data/iceland-surface.grib");
///let product_kind = ProductKind::GRIB;
///
///let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();
///
///// Print names of messages in the file
///while let Some(message) = handle.next().unwrap() {
///// The message must be unwraped as internal Iterator methods can fail
///    let key = message.read_key("name").unwrap();
///
///    if let KeyType::Str(name) = key.value {
///        println!("{:?}", name);    
///    }
///}
///```
///
///The `FallibleIterator` can be collected to convert the handle into a
///`Vector` of `KeyedMessage`s.
///For example:
///
///```
///# use eccodes::codes_handle::{ProductKind, CodesHandle, KeyedMessage};
///# use eccodes::errors::CodesError;
///# use std::path::Path;
///# use fallible_iterator::FallibleIterator;
///#
///let file_path = Path::new("./data/iceland-surface.grib");
///let product_kind = ProductKind::GRIB;
///
///let handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();
///
///let handle_collected: Vec<KeyedMessage> = handle.collect().unwrap();
///```
///
///Use of `filter()`, `map()` and other methods provided with `Iterator` allow for
///more advanced extracting of GRIB messages from the file.
///
///## Errors
///The `next()` method will return [`CodesInternal`](crate::errors::CodesInternal)
///when internal ecCodes function returns non-zero code.
///
impl FallibleIterator for CodesHandle {
    type Item = KeyedMessage;

    type Error = CodesError;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        let file_handle;
        unsafe {
            file_handle = codes_handle_new_from_file(self.file_pointer, self.product_kind);
        }

        match file_handle {
            Ok(h) => {
                self.file_handle = h;

                if self.file_handle.is_null() {
                    Ok(None)
                } else {
                    let message = get_message_from_handle(h)?;
                    Ok(Some(message))
                }
            }
            Err(e) => Err(e),
        }
    }
}

fn get_message_from_handle(handle: *mut codes_handle) -> Result<KeyedMessage, CodesError> {
    let new_handle;

    unsafe {
        let message_tuple = codes_get_message(handle)?;
        new_handle = codes_handle_new_from_message(message_tuple.0, message_tuple.1);
    }

    let new_message = KeyedMessage {
        message_handle: new_handle,
        message_buffer: vec![],
        iterator_flags: None,
        iterator_namespace: None,
        keys_iterator: None,
        keys_iterator_next_time_exists: false,
    };

    Ok(new_message)
}

#[cfg(test)]
mod tests {
    use crate::codes_handle::{CodesHandle, KeyType, KeyedMessage, ProductKind};
    use fallible_iterator::FallibleIterator;
    use std::path::Path;

    #[test]
    fn iterator_fn() {
        let file_path = Path::new("./data/iceland-surface.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();

        while let Some(msg) = handle.next().unwrap() {
            let key = msg.read_key("shortName").unwrap();

            match key.value {
                KeyType::Str(_) => {}
                _ => panic!("Incorrect variant of string key"),
            }
        }

        let handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();

        let handle_collected: Vec<KeyedMessage> = handle.collect().unwrap();

        for msg in handle_collected {
            let key = msg.read_key("name").unwrap();
            match key.value {
                KeyType::Str(_) => {}
                _ => panic!("Incorrect variant of string key"),
            }
        }
    }
}
