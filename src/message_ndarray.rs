use ndarray::{s, Array2, Array3};

use crate::{errors::MessageNdarrayError, CodesError, KeyType, KeyedMessage};

impl KeyedMessage {
    /// Returns [y, x] ([Nj, Ni], [lat, lon]) ndarray from the message,
    /// x coordinates are increasing with the i index,
    /// y coordinates are decreasing with the j index.
    pub fn to_ndarray(&self) -> Result<Array2<f64>, CodesError> {
        let ni = if let KeyType::Int(ni) = self.read_key("Ni")?.value {
            ni
        } else {
            return Err(MessageNdarrayError::UnexpectedKeyType("Ni".to_owned()).into());
        };

        let nj = if let KeyType::Int(nj) = self.read_key("Nj")?.value {
            nj
        } else {
            return Err(MessageNdarrayError::UnexpectedKeyType("Nj".to_owned()).into());
        };

        let vals = if let KeyType::FloatArray(vals) = self.read_key("values")?.value {
            vals
        } else {
            return Err(MessageNdarrayError::UnexpectedKeyType("values".to_owned()).into());
        };

        if vals.len() != (ni * nj) as usize {
            return Err(MessageNdarrayError::UnexpectedValuesLength(vals.len(), ni * nj).into());
        }

        let shape = (nj as usize, ni as usize);
        let vals = Array2::from_shape_vec(shape, vals).map_err(|e| MessageNdarrayError::from(e))?;

        Ok(vals)
    }

    pub fn to_lons_lats_values(
        &self,
    ) -> Result<(Array2<f64>, Array2<f64>, Array2<f64>), CodesError> {
        let ni = if let KeyType::Int(ni) = self.read_key("Ni")?.value {
            ni
        } else {
            return Err(MessageNdarrayError::UnexpectedKeyType("Ni".to_owned()).into());
        };

        let nj = if let KeyType::Int(nj) = self.read_key("Nj")?.value {
            nj
        } else {
            return Err(MessageNdarrayError::UnexpectedKeyType("Nj".to_owned()).into());
        };

        let latlonvals = if let KeyType::FloatArray(vals) = self.read_key("latLonValues")?.value {
            vals
        } else {
            return Err(MessageNdarrayError::UnexpectedKeyType("values".to_owned()).into());
        };

        if latlonvals.len() != (ni * nj * 3) as usize {
            return Err(
                MessageNdarrayError::UnexpectedValuesLength(latlonvals.len(), ni * nj * 3).into(),
            );
        }

        let shape = (nj as usize, ni as usize, 3_usize);
        let mut latlonvals =
            Array3::from_shape_vec(shape, latlonvals).map_err(|e| MessageNdarrayError::from(e))?;
        let (lats, lons, vals) =
            latlonvals
                .view_mut()
                .multi_slice_move((s![.., .., 0], s![.., .., 1], s![.., .., 2]));

        Ok((lons.into_owned(), lats.into_owned(), vals.into_owned()))
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
                let (lats, lons, vals) = msg.to_lons_lats_values()?;

                // values from xarray
                assert_approx_eq!(f64, vals[[0, 0]], 276.37793, epsilon = 0.000_1);
                assert_approx_eq!(f64, vals[[0, 48]], 276.65723, epsilon = 0.000_1);
                assert_approx_eq!(f64, vals[[16, 0]], 277.91113, epsilon = 0.000_1);
                assert_approx_eq!(f64, vals[[16, 48]], 280.34277, epsilon = 0.000_1);
                assert_approx_eq!(f64, vals[[5, 5]], 276.03418, epsilon = 0.000_1);
                assert_approx_eq!(f64, vals[[10, 10]], 277.59082, epsilon = 0.000_1);
                assert_approx_eq!(f64, vals[[15, 15]], 277.68652, epsilon = 0.000_1);
                assert_approx_eq!(f64, vals[[8, 37]], 273.2744, epsilon = 0.000_1);

                break;
            }
        }

        todo!("Test lats and lons")
    }
}
