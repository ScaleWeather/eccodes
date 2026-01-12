//! Definition and associated functions of `CodesNearest`
//! used for finding nearest gridpoints in `CodesMessage`

use std::{fmt::Debug, ptr::null_mut};

use eccodes_sys::codes_nearest;
use tracing::{Level, event, instrument};

use crate::{
    CodesError,
    codes_message::CodesMessage,
    intermediate_bindings::{
        codes_grib_nearest_delete, codes_grib_nearest_find, codes_grib_nearest_new,
    },
};

/// The structure used to find nearest gridpoints in `CodesMessage`.
#[derive(Debug)]
pub struct CodesNearest<'a, P: Debug> {
    nearest_handle: *mut codes_nearest,
    parent_message: &'a CodesMessage<P>,
}

/// The structure returned by [`CodesNearest::find_nearest()`].
/// Should always be analysed in relation to the coordinates requested in `find_nearest()`.
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct NearestGridpoint {
    ///Index of this gridpoint
    pub index: i32,
    ///Latitude of this gridpoint in degrees north
    pub lat: f64,
    ///Longitude of this gridpoint in degrees east
    pub lon: f64,
    /// Distance between requested point and this gridpoint in kilometers
    pub distance: f64,
    ///Value of parameter at this gridpoint contained by `CodesMessage` in corresponding units
    pub value: f64,
}

impl<P: Debug> CodesMessage<P> {
    /// Creates a new instance of [`CodesNearest`] for the `CodesMessage`.
    /// [`CodesNearest`] can be used to find nearest gridpoints for given coordinates in the `CodesMessage`
    /// by calling [`find_nearest()`](crate::CodesNearest::find_nearest).
    ///
    /// # Errors
    ///
    /// This function returns [`CodesInternal`](crate::errors::CodesInternal) when
    /// internal nearest handle cannot be created.
    pub fn codes_nearest(&self) -> Result<CodesNearest<'_, P>, CodesError> {
        let nearest_handle = unsafe { codes_grib_nearest_new(self.message_handle)? };

        Ok(CodesNearest {
            nearest_handle,
            parent_message: self,
        })
    }
}

impl<P: Debug> CodesNearest<'_, P> {
    ///Function to get four [`NearestGridpoint`]s of a point represented by requested coordinates.
    ///
    ///The inputs are latitude and longitude of requested point in respectively degrees north and
    ///degreed east.
    ///
    ///### Example
    ///
    ///```
    /// # use anyhow::Context;
    /// # use eccodes::{CodesFile, FallibleIterator, ProductKind};
    /// # fn main() -> anyhow::Result<()> {
    /// let mut handle = CodesFile::new_from_file("./data/iceland.grib", ProductKind::GRIB)?;
    /// let msg = handle.ref_message_iter().next()?.context("no message")?;
    ///
    /// let mut c_nearest = msg.codes_nearest()?;
    /// let out = c_nearest.find_nearest(64.13, -21.89)?;
    /// #     Ok(())
    /// # }
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

impl<P: Debug> Drop for CodesNearest<'_, P> {
    /// # Panics
    ///
    /// In debug when error is encountered
    #[instrument(level = "trace")]
    fn drop(&mut self) {
        unsafe {
            codes_grib_nearest_delete(self.nearest_handle).unwrap_or_else(|error| {
                event!(
                    Level::ERROR,
                    "codes_grib_nearest_delete() returned an error: {:?}",
                    &error
                );
                debug_assert!(false, "Error in CodesNearest::drop");
            });
        }

        self.nearest_handle = null_mut();
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use anyhow::{Context, Result};
    use fallible_iterator::FallibleIterator;

    use crate::{CodesFile, ProductKind};

    #[test]
    fn find_nearest() -> Result<()> {
        let file_path1 = Path::new("./data/iceland.grib");
        let file_path2 = Path::new("./data/iceland-surface.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle1 = CodesFile::new_from_file(file_path1, product_kind)?;
        let msg1 = handle1
            .ref_message_iter()
            .next()?
            .context("Message not some")?;
        let mut nrst1 = msg1.codes_nearest()?;
        let out1 = nrst1.find_nearest(64.13, -21.89)?;

        let mut handle2 = CodesFile::new_from_file(file_path2, product_kind)?;
        let msg2 = handle2
            .ref_message_iter()
            .next()?
            .context("Message not some")?;
        let mut nrst2 = msg2.codes_nearest()?;
        let out2 = nrst2.find_nearest(64.13, -21.89)?;

        assert!(out1[0].value > 10000.0);
        assert!(out2[3].index == 551);
        assert!((out1[1].lat - 64.0).abs() < f64::EPSILON);
        assert!((out2[2].lon - -21.75).abs() < f64::EPSILON);
        assert!(out1[0].distance > 15.0);

        Ok(())
    }

    #[test]
    fn destructor() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;
        let current_message = handle
            .ref_message_iter()
            .next()?
            .context("Message not some")?;

        let _nrst = current_message.codes_nearest()?;

        Ok(())
    }

    #[test]
    fn destructor_null() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;
        let current_message = handle
            .ref_message_iter()
            .next()?
            .context("Message not some")?;

        let mut nrst = current_message.codes_nearest()?;
        nrst.nearest_handle = std::ptr::null_mut();

        Ok(())
    }
}
