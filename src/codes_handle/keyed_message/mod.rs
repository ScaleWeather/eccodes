mod iterator;
mod read;
mod write;

use eccodes_sys::codes_nearest;
use log::warn;
use std::ptr::null_mut;

use crate::{
    codes_handle::KeyedMessage,
    errors::CodesError,
    intermediate_bindings::{
        codes_get_message_copy, codes_grib_nearest_delete, codes_grib_nearest_find,
        codes_grib_nearest_new, codes_handle_delete, codes_handle_new_from_message_copy,
        codes_keys_iterator_delete,
    },
};

use super::{KeysIteratorFlags, NearestGridpoint};

impl KeyedMessage {
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
    ///# use eccodes::FallibleIterator;
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
            iterator_flags: None,
            iterator_namespace: None,
            keys_iterator: None,
            keys_iterator_next_item_exists: false,
            nearest_handle: None,
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
    ///The pointers are cleared at the end of drop as they are not functional despite the result of delete functions.
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

        self.nearest_handle = Some(null_mut());

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
    use crate::codes_handle::{CodesHandle, ProductKind};
    use crate::FallibleIterator;
    use std::path::Path;
    use testing_logger;

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
        assert!(cloned_message.iterator_flags.is_none());
        assert!(cloned_message.iterator_namespace.is_none());
        assert!(cloned_message.keys_iterator.is_none());
        assert!(!cloned_message.keys_iterator_next_item_exists);
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
}
