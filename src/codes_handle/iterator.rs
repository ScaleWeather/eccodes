use eccodes_sys::codes_handle;
use fallible_iterator::FallibleIterator;

use crate::{
    codes_handle::{CodesHandle, KeyedMessage},
    errors::CodesError,
    intermediate_bindings::{
        codes_get_message_copy, codes_handle_delete, codes_handle_new_from_file,
        codes_handle_new_from_message_copy, codes_index::codes_iter_next_from_index,
    },
    CodesIndex,
};

use super::GribFile;

///`FallibleIterator` implementation for `CodesHandle` to access GRIB messages inside file.
///
///To access GRIB messages the ecCodes library uses a method similar to a C-style iterator.
///It digests the `FILE *` multiple times each time returning the `codes_handle` raw pointer
///to a message inside the file. This method would be unsafe to expose directly.
///Therefore this crate utilizes the `Iterator` to provide the access to GRIB messages in
///a safe and convienient way.
///
///[`FallibleIterator`] is used instead of classic `Iterator`
///because internal ecCodes functions can return error codes when the GRIB file
///is corrupted and for some other reasons. The usage of `FallibleIterator` is sligthly different
///than usage of `Iterator`, check its documentation for more details.
///
///For a true memory safety and to provide a ful Rust Iterator functionality,
///this iterator clones each message to a new buffer.Although internal ecCodes
///message copy implementation makes this operation quite cheap, using this iterator
///(and in effect this crate) comes with memory overhead, but is
///a necessity for memory safety.
///
///Using the `FallibleIterator` is the only way to read `KeyedMessage`s from the file.
///Its basic usage is simply with while let statement (similar to for loop for classic `Iterator`):
///
///```
///# use eccodes::codes_handle::{ProductKind, CodesHandle, KeyType};
///# use std::path::Path;
///# use eccodes::FallibleIterator;
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
///# use eccodes::FallibleIterator;
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
impl FallibleIterator for CodesHandle<GribFile> {
    type Item = KeyedMessage;

    type Error = CodesError;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        let new_eccodes_handle;
        unsafe {
            codes_handle_delete(self.eccodes_handle)?;
            new_eccodes_handle = codes_handle_new_from_file(self.source.pointer, self.product_kind);
        }

        match new_eccodes_handle {
            Ok(h) => {
                self.eccodes_handle = h;

                if self.eccodes_handle.is_null() {
                    Ok(None)
                } else {
                    let message = get_message_from_handle(h);
                    Ok(Some(message))
                }
            }
            Err(e) => Err(e),
        }
    }
}

impl FallibleIterator for CodesHandle<CodesIndex> {
    type Item = KeyedMessage;

    type Error = CodesError;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        let new_eccodes_handle;
        unsafe {
            codes_handle_delete(self.eccodes_handle)?;
            new_eccodes_handle = codes_iter_next_from_index(self.source.pointer);
        }

        match new_eccodes_handle {
            Ok(h) => {
                self.eccodes_handle = h;

                if self.eccodes_handle.is_null() {
                    Ok(None)
                } else {
                    let message = get_message_from_handle(h);
                    Ok(Some(message))
                }
            }
            Err(e) => Err(e),
        }
    }
}

fn get_message_from_handle(handle: *mut codes_handle) -> KeyedMessage {
    let new_handle;
    let new_buffer;

    unsafe {
        new_buffer = codes_get_message_copy(handle).expect(
            "Getting message clone failed.
        Please report this panic on Github",
        );
        new_handle = codes_handle_new_from_message_copy(&new_buffer);
    }

    KeyedMessage {
        message_handle: new_handle,
        iterator_flags: None,
        iterator_namespace: None,
        keys_iterator: None,
        keys_iterator_next_item_exists: false,
        nearest_handle: None,
    }
}

#[cfg(test)]
mod tests {
    use crate::codes_handle::{CodesHandle, KeyType, KeyedMessage, ProductKind};
    use crate::FallibleIterator;
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

    #[test]
    fn iterator_return() {
        let file_path = Path::new("./data/iceland-surface.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();
        let current_message = handle.next().unwrap().unwrap();

        assert!(!current_message.message_handle.is_null());
        assert!(current_message.iterator_flags.is_none());
        assert!(current_message.iterator_namespace.is_none());
        assert!(current_message.keys_iterator.is_none());
        assert!(!current_message.keys_iterator_next_item_exists);
    }

    #[test]
    fn iterator_collect() {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();

        // Use iterator to get a Keyed message with shortName "msl" and typeOfLevel "surface"
        // First, filter and collect the messages to get those that we want
        let mut level: Vec<KeyedMessage> = handle
            .filter(|msg| {
                Ok(
                    msg.read_key("shortName")?.value == KeyType::Str("msl".to_string())
                        && msg.read_key("typeOfLevel")?.value
                            == KeyType::Str("surface".to_string()),
                )
            })
            .collect()
            .unwrap();

        // Now unwrap and access the first and only element of resulting vector
        // Find nearest modifies internal KeyedMessage fields so we need mutable reference
        let level = &mut level[0];

        println!("{:?}", level.read_key("shortName"));

        // Get the four nearest gridpoints of Reykjavik
        let nearest_gridpoints = level.find_nearest(64.13, -21.89).unwrap();

        // Print value and distance of the nearest gridpoint
        println!(
            "value: {}, distance: {}",
            nearest_gridpoints[3].value, nearest_gridpoints[3].distance
        );
    }
}
