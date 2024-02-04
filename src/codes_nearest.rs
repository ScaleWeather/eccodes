use std::ptr::null_mut;

use eccodes_sys::codes_nearest;
use log::warn;

use crate::{
    intermediate_bindings::{
        codes_grib_nearest_delete, codes_grib_nearest_find, codes_grib_nearest_new,
    },
    CodesError, KeyedMessage,
};

#[derive(Debug)]
pub struct CodesNearest<'a> {
    nearest_handle: *mut codes_nearest,
    parent_message: &'a KeyedMessage,
}

///The structure returned by [`KeyedMessage::find_nearest()`].
///Should always be analysed in relation to the coordinates request in `find_nearest()`.
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct NearestGridpoint {
    ///Index of gridpoint
    pub index: i32,
    ///Latitude in degrees north
    pub lat: f64,
    ///Longitude in degrees east
    pub lon: f64,
    ///Distance from coordinates requested in `find_nearest()`
    pub distance: f64,
    ///Value of the filed at given coordinate
    pub value: f64,
}

impl KeyedMessage {
    pub fn codes_nearest(&self) -> Result<CodesNearest, CodesError> {
        let nearest_handle = unsafe { codes_grib_nearest_new(self.message_handle)? };

        Ok(CodesNearest {
            nearest_handle,
            parent_message: self,
        })
    }
}

impl CodesNearest<'_> {
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
    pub fn find_nearest(&self, lat: f64, lon: f64) -> Result<[NearestGridpoint; 4], CodesError> {
        let output_points;

        unsafe {
            output_points = codes_grib_nearest_find(
                self.parent_message.message_handle,
                self.nearest_handle,
                lat,
                lon,
            )?;
        }

        Ok(output_points)
    }
}

impl Drop for CodesNearest<'_> {
    fn drop(&mut self) {
        unsafe {
            codes_grib_nearest_delete(self.nearest_handle).unwrap_or_else(|error| {
                warn!(
                    "codes_grib_nearest_delete() returned an error: {:?}",
                    &error
                );
            });
        }

        self.nearest_handle = null_mut();
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use anyhow::{Context, Result};
    use fallible_streaming_iterator::FallibleStreamingIterator;

    use crate::{CodesHandle, ProductKind};

    #[test]
    fn find_nearest() -> Result<()> {
        let file_path1 = Path::new("./data/iceland.grib");
        let file_path2 = Path::new("./data/iceland-surface.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle1 = CodesHandle::new_from_file(file_path1, product_kind)?;
        let msg1 = handle1.next()?.context("Message not some")?;
        let nrst1 = msg1.codes_nearest()?;
        let out1 = nrst1.find_nearest(64.13, -21.89)?;

        let mut handle2 = CodesHandle::new_from_file(file_path2, product_kind)?;
        let msg2 = handle2.next()?.context("Message not some")?;
        let nrst2 = msg2.codes_nearest()?;
        let out2 = nrst2.find_nearest(64.13, -21.89)?;

        assert!(out1[0].value > 10000.0);
        assert!(out2[3].index == 551);
        assert!(out1[1].lat == 64.0);
        assert!(out2[2].lon == -21.75);
        assert!(out1[0].distance > 15.0);

        Ok(())
    }
}
