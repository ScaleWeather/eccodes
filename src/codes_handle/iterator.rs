use crate::{
    codes_handle::{CodesHandle, KeyedMessage},
    errors::CodesError,
    intermediate_bindings::codes_handle_new_from_file,
};

///Iterator implementation for `CodesHandle` to access GRIB messages inside file.
///
///To access GRIB messages the ecCodes library uses a method similar to a C-style iterator.
///It digests the `FILE *` multiple times each time returning the `codes_handle` raw pointer
///to a message inside the file. This method would be unsafe to expose directly.
///Therefore this crate utilizes the `Iterator` to provide the access to GRIB messages in
///a safe and convienient way.
///
///Using the `Iterator` is the only way to read `KeyedMessage`s from the file.
///Its basic usage is simply with for loop:
///
///```
///# use eccodes::codes_handle::{ProductKind, CodesHandle, Key};
///# use std::path::Path;
///#
///let file_path = Path::new("./data/iceland-surface.grib");
///let product_kind = ProductKind::GRIB;
///
///let handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();
///
///// Print names of messages in the file
///for message in handle {
///// The message must be unwraped as internal Iterator methods can fail
///    let message = message.unwrap();
///    let key = message.read_key("name").unwrap();
///
///    if let Key::Str(name) = key {
///        println!("{:?}", name);    
///    }
///}
///```
///
///The `Iterator` can be collected to convert the handle into a 
///`Vector` of `KeyedMessage`s without a memory overhead (`KeyedMessages`
///are kept as pointers).
///For example:
///
///```
///# use eccodes::codes_handle::{ProductKind, CodesHandle, Key, KeyedMessage};
///# use eccodes::errors::CodesError;
///# use std::path::Path;
///#
///let file_path = Path::new("./data/iceland-surface.grib");
///let product_kind = ProductKind::GRIB;
///
///let handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();
///
///let handle_collected: Result<Vec<KeyedMessage>, CodesError> = handle.collect();
///let handle_collected: Vec<KeyedMessage> = handle_collected.unwrap();
///```
///
///Use of `filter()`, `map()` and other methods provided with `Iterator` allow for
///more advanced extracting of GRIB messages from the file.
///
///## Errors
///The `next()` method will return [`CodesInternal`](crate::errors::CodesInternal)
///when internal ecCodes function returns non-zero code.
///
impl Iterator for CodesHandle {
    type Item = Result<KeyedMessage, CodesError>;

    fn next(&mut self) -> Option<Self::Item> {
        let file_handle;
        unsafe {
            file_handle = codes_handle_new_from_file(self.file_pointer, self.product_kind);
        }

        match file_handle {
            Ok(h) => {
                self.current_message.file_handle = h;

                if self.current_message.file_handle.is_null() {
                    None
                } else {
                    Some(Ok(self.current_message))
                }
            }
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        codes_handle::{CodesHandle, Key, KeyedMessage, ProductKind},
        errors::CodesError,
    };
    use std::path::Path;

    #[test]
    fn iterator_fn() {
        let file_path = Path::new("./data/iceland-surface.grib");
        let product_kind = ProductKind::GRIB;

        let handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();

        for msg in handle {
            let msg = msg.unwrap();

            let key = msg.read_key("shortName").unwrap();

            match key {
                Key::Str(_) => {}
                _ => panic!("Incorrect variant of string key"),
            }
        }

        let handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();

        let handle_collected: Result<Vec<KeyedMessage>, CodesError> = handle.collect();
        let handle_collected = handle_collected.unwrap();

        for msg in handle_collected {
            let key = msg.read_key("name").unwrap();
            match key {
                Key::Str(_) => {}
                _ => panic!("Incorrect variant of string key"),
            }
        }
    }
}
