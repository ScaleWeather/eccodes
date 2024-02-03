use std::ptr;

use fallible_streaming_iterator::FallibleStreamingIterator;

use crate::{
    codes_handle::{CodesHandle, KeyedMessage},
    errors::CodesError,
    intermediate_bindings::{codes_handle_delete, codes_handle_new_from_file},
};
#[cfg(feature = "experimental_index")]
use crate::{intermediate_bindings::codes_index::codes_handle_new_from_index, CodesIndex};

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
impl FallibleStreamingIterator for CodesHandle<GribFile> {
    type Item = KeyedMessage;

    type Error = CodesError;

    fn advance(&mut self) -> Result<(), Self::Error> {
        unsafe {
            codes_handle_delete(self.unsafe_message.message_handle)?;
        }

        // nullify message handle so that destructor is harmless
        // it might be excessive but it follows the correct pattern
        self.unsafe_message.message_handle = ptr::null_mut();

        let new_eccodes_handle =
            unsafe { codes_handle_new_from_file(self.source.pointer, self.product_kind)? };

        self.unsafe_message = KeyedMessage {
            message_handle: new_eccodes_handle,
            iterator_flags: None,
            iterator_namespace: None,
            keys_iterator: None,
            keys_iterator_next_item_exists: false,
            nearest_handle: None,
        };

        Ok(())
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.unsafe_message.message_handle.is_null() {
            None
        } else {
            Some(&self.unsafe_message)
      }
    }
}

#[cfg(feature = "experimental_index")]
impl FallibleStreamingIterator for CodesHandle<CodesIndex> {
    type Item = KeyedMessage;

    type Error = CodesError;

    fn advance(&mut self) -> Result<(), Self::Error> {
        unsafe {
            codes_handle_delete(self.unsafe_message.message_handle)?;
        }

        // nullify message handle so that destructor is harmless
        // it might be excessive but it follows the correct pattern
        self.unsafe_message.message_handle = ptr::null_mut();

        let new_eccodes_handle = unsafe { codes_handle_new_from_index(self.source.pointer)? };

        self.unsafe_message = KeyedMessage {
            message_handle: new_eccodes_handle,
            iterator_flags: None,
            iterator_namespace: None,
            keys_iterator: None,
            keys_iterator_next_item_exists: false,
            nearest_handle: None,
        };

        Ok(())
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.unsafe_message.message_handle.is_null() {
            None
        } else {
            Some(&self.unsafe_message)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::codes_handle::{CodesHandle, KeyType, ProductKind};
    use anyhow::Result;
    use fallible_streaming_iterator::FallibleStreamingIterator;
    use std::path::Path;

    #[test]
    fn iterator_lifetimes() -> Result<()> {
        let file_path = Path::new("./data/iceland-levels.grib");
        let product_kind = ProductKind::GRIB;
        let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();

        let msg1 = handle.next()?.unwrap();
        let key1 = msg1.read_key("typeOfLevel")?;

        let msg2 = handle.next()?.unwrap();
        let key2 = msg2.read_key("typeOfLevel")?;

        let msg3 = handle.next()?.unwrap();
        let key3 = msg3.read_key("typeOfLevel")?;

        assert_eq!(key1.value, KeyType::Str("isobaricInhPa".to_string()));
        assert_eq!(key2.value, KeyType::Str("isobaricInhPa".to_string()));
        assert_eq!(key3.value, KeyType::Str("isobaricInhPa".to_string()));

        Ok(())
    }

    #[test]
    fn iterator_fn() -> Result<()> {
        let file_path = Path::new("./data/iceland-surface.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();

        while let Some(msg) = handle.next()? {
            let key = msg.read_key("shortName")?;

            match key.value {
                KeyType::Str(_) => {}
                _ => panic!("Incorrect variant of string key"),
            }
        }

        Ok(())
    }

    #[test]
    fn iterator_collected() -> Result<()> {
        let file_path = Path::new("./data/iceland-surface.grib");
        let product_kind = ProductKind::GRIB;
        let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();

        let mut handle_collected = vec![];

        while let Some(msg) = handle.next()? {
            handle_collected.push(msg.clone());
        }

        for msg in handle_collected {
            let key = msg.read_key("name").unwrap();
            match key.value {
                KeyType::Str(_) => {}
                _ => panic!("Incorrect variant of string key"),
            }
        }

        Ok(())
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
    fn iterator_filter() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();

        // Use iterator to get a Keyed message with shortName "msl" and typeOfLevel "surface"
        // First, filter and collect the messages to get those that we want
        let mut level = vec![];

        while let Some(msg) = handle.next()? {
            if msg.read_key("shortName")?.value == KeyType::Str("msl".to_string())
                && msg.read_key("typeOfLevel")?.value == KeyType::Str("surface".to_string())
            {
                level.push(msg.clone());
            }
        }

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

        Ok(())
    }
}
