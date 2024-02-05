use ndarray::{s, Array2, Array3};

use crate::{errors::MessageNdarrayError, CodesError, KeyType, KeyedMessage};

#[derive(Clone, PartialEq, Debug, Default)]
pub struct RustyCodesMessage {
    pub longitudes: Array2<f64>,
    pub latitudes: Array2<f64>,
    pub values: Array2<f64>,
}

impl KeyedMessage {
    /// Returns [y, x] ([Nj, Ni], [lat, lon]) ndarray from the message,
    /// x coordinates are increasing with the i index,
    /// y coordinates are decreasing with the j index.
    #[cfg_attr(docsrs, doc(cfg(feature = "message_ndarray")))]
    pub fn to_ndarray(&self) -> Result<Array2<f64>, CodesError> {
        let KeyType::Int(ni) = self.read_key("Ni")?.value else {
            return Err(MessageNdarrayError::UnexpectedKeyType("Ni".to_owned()).into());
        };

        let KeyType::Int(nj) = self.read_key("Nj")?.value else {
            return Err(MessageNdarrayError::UnexpectedKeyType("Nj".to_owned()).into());
        };

        let KeyType::FloatArray(vals) = self.read_key("values")?.value else {
            return Err(MessageNdarrayError::UnexpectedKeyType("values".to_owned()).into());
        };

        let ni = usize::try_from(ni).map_err(MessageNdarrayError::from)?;
        let nj = usize::try_from(nj).map_err(MessageNdarrayError::from)?;

        if vals.len() != (ni * nj) {
            return Err(MessageNdarrayError::UnexpectedValuesLength(vals.len(), ni * nj).into());
        }

        let shape = (nj, ni);
        let vals = Array2::from_shape_vec(shape, vals).map_err(MessageNdarrayError::from)?;

        Ok(vals)
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "message_ndarray")))]
    pub fn to_lons_lats_values(&self) -> Result<RustyCodesMessage, CodesError> {
        let KeyType::Int(ni) = self.read_key("Ni")?.value else {
            return Err(MessageNdarrayError::UnexpectedKeyType("Ni".to_owned()).into());
        };

        let KeyType::Int(nj) = self.read_key("Nj")?.value else {
            return Err(MessageNdarrayError::UnexpectedKeyType("Nj".to_owned()).into());
        };

        let KeyType::FloatArray(latlonvals) = self.read_key("latLonValues")?.value else {
            return Err(MessageNdarrayError::UnexpectedKeyType("latLonValues".to_owned()).into());
        };

        let ni = usize::try_from(ni).map_err(MessageNdarrayError::from)?;
        let nj = usize::try_from(nj).map_err(MessageNdarrayError::from)?;

        if latlonvals.len() != (ni * nj * 3) {
            return Err(
                MessageNdarrayError::UnexpectedValuesLength(latlonvals.len(), ni * nj * 3).into(),
            );
        }

        let shape = (nj, ni, 3_usize);
        let mut latlonvals =
            Array3::from_shape_vec(shape, latlonvals).map_err(MessageNdarrayError::from)?;
        let (lats, lons, vals) =
            latlonvals
                .view_mut()
                .multi_slice_move((s![.., .., 0], s![.., .., 1], s![.., .., 2]));

        Ok(RustyCodesMessage {
            longitudes: lons.into_owned(),
            latitudes: lats.into_owned(),
            values: vals.into_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use float_cmp::assert_approx_eq;

    use super::*;
    use crate::codes_handle::CodesHandle;
    use crate::FallibleStreamingIterator;
    use crate::KeyType;
    use crate::ProductKind;
    use std::path::Path;

    #[test]
    fn test_to_ndarray() -> Result<(), CodesError> {
        let file_path = Path::new("./data/iceland-surface.grib");
        let mut handle = CodesHandle::new_from_file(file_path, ProductKind::GRIB)?;

        while let Some(msg) = handle.next()? {
            if msg.read_key("shortName")?.value == KeyType::Str("2d".to_string()) {
                let ndarray = msg.to_ndarray()?;

                // values from xarray
                assert_approx_eq!(f64, ndarray[[0, 0]], 276.37793, epsilon = 0.000_1);
                assert_approx_eq!(f64, ndarray[[0, 48]], 276.65723, epsilon = 0.000_1);
                assert_approx_eq!(f64, ndarray[[16, 0]], 277.91113, epsilon = 0.000_1);
                assert_approx_eq!(f64, ndarray[[16, 48]], 280.34277, epsilon = 0.000_1);
                assert_approx_eq!(f64, ndarray[[5, 5]], 276.03418, epsilon = 0.000_1);
                assert_approx_eq!(f64, ndarray[[10, 10]], 277.59082, epsilon = 0.000_1);
                assert_approx_eq!(f64, ndarray[[15, 15]], 277.68652, epsilon = 0.000_1);
                assert_approx_eq!(f64, ndarray[[8, 37]], 273.2744, epsilon = 0.000_1);

                break;
            }
        }

        Ok(())
    }

    #[test]
    fn test_lons_lats() -> Result<(), CodesError> {
        let file_path = Path::new("./data/iceland-surface.grib");
        let mut handle = CodesHandle::new_from_file(file_path, ProductKind::GRIB)?;

        while let Some(msg) = handle.next()? {
            if msg.read_key("shortName")?.value == KeyType::Str("2d".to_string()) {
                let rmsg = msg.to_lons_lats_values()?;

                let vals = rmsg.values;
                let lons = rmsg.longitudes;
                let lats = rmsg.latitudes;

                // values from cfgrib
                assert_approx_eq!(f64, vals[[0, 0]], 276.37793, epsilon = 0.000_1);
                assert_approx_eq!(f64, vals[[0, 48]], 276.65723, epsilon = 0.000_1);
                assert_approx_eq!(f64, vals[[16, 0]], 277.91113, epsilon = 0.000_1);
                assert_approx_eq!(f64, vals[[16, 48]], 280.34277, epsilon = 0.000_1);
                assert_approx_eq!(f64, vals[[5, 5]], 276.03418, epsilon = 0.000_1);
                assert_approx_eq!(f64, vals[[10, 10]], 277.59082, epsilon = 0.000_1);
                assert_approx_eq!(f64, vals[[15, 15]], 277.68652, epsilon = 0.000_1);
                assert_approx_eq!(f64, vals[[8, 37]], 273.2744, epsilon = 0.000_1);

                assert_approx_eq!(f64, lons[[0, 0]], -25.0);
                assert_approx_eq!(f64, lons[[0, 48]], -13.0);
                assert_approx_eq!(f64, lons[[16, 0]], -25.0);
                assert_approx_eq!(f64, lons[[16, 48]], -13.0);
                assert_approx_eq!(f64, lons[[5, 5]], -23.75);
                assert_approx_eq!(f64, lons[[10, 10]], -22.5);
                assert_approx_eq!(f64, lons[[15, 15]], -21.25);
                assert_approx_eq!(f64, lons[[8, 37]], -15.75);

                assert_approx_eq!(f64, lats[[0, 0]], 67.0);
                assert_approx_eq!(f64, lats[[0, 48]], 67.0);
                assert_approx_eq!(f64, lats[[16, 0]], 63.0);
                assert_approx_eq!(f64, lats[[16, 48]], 63.0);
                assert_approx_eq!(f64, lats[[5, 5]], 65.75);
                assert_approx_eq!(f64, lats[[10, 10]], 64.5);
                assert_approx_eq!(f64, lats[[15, 15]], 63.25);
                assert_approx_eq!(f64, lats[[8, 37]], 65.0);

                break;
            }
        }

        Ok(())
    }
}
