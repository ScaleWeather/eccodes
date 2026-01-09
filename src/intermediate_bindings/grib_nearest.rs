#![allow(non_camel_case_types)]
#![allow(clippy::module_name_repetitions)]

use std::ptr::addr_of_mut;

use eccodes_sys::{CODES_NEAREST_SAME_DATA, CODES_NEAREST_SAME_GRID, codes_handle, codes_nearest};

use tracing::instrument;

use crate::{
    NearestGridpoint, errors::CodesError, intermediate_bindings::error_code_to_result,
    pointer_guard,
};

#[instrument(level = "trace")]
pub unsafe fn codes_grib_nearest_new(
    handle: *const codes_handle,
) -> Result<*mut codes_nearest, CodesError> {
    unsafe {
        pointer_guard::non_null!(handle);

        let mut error_code: i32 = 0;

        let nearest = eccodes_sys::codes_grib_nearest_new(handle, &raw mut error_code);
        error_code_to_result(error_code)?;

        Ok(nearest)
    }
}

#[instrument(level = "trace")]
pub unsafe fn codes_grib_nearest_delete(nearest: *mut codes_nearest) -> Result<(), CodesError> {
    unsafe {
        if nearest.is_null() {
            return Ok(());
        }

        let error_code = eccodes_sys::codes_grib_nearest_delete(nearest);
        error_code_to_result(error_code)?;

        Ok(())
    }
}

pub unsafe fn codes_grib_nearest_find(
    handle: *const codes_handle,
    nearest: *mut codes_nearest,
    lat: f64,
    lon: f64,
) -> Result<[NearestGridpoint; 4], CodesError> {
    unsafe {
        pointer_guard::non_null!(handle);
        pointer_guard::non_null!(nearest);

        // such flags are set because find nearest for given nearest is always
        // called on the same grib message
        let flags = CODES_NEAREST_SAME_GRID + CODES_NEAREST_SAME_DATA;

        let mut output_lats = [0_f64; 4];
        let mut output_lons = [0_f64; 4];
        let mut output_values = [0_f64; 4];
        let mut output_distances = [0_f64; 4];
        let mut output_indexes = [0_i32; 4];

        let mut length: usize = 4;

        let error_code = eccodes_sys::codes_grib_nearest_find(
            nearest,
            handle,
            lat,
            lon,
            u64::from(flags),
            addr_of_mut!(output_lats[0]),
            addr_of_mut!(output_lons[0]),
            addr_of_mut!(output_values[0]),
            addr_of_mut!(output_distances[0]),
            addr_of_mut!(output_indexes[0]),
            &raw mut length,
        );
        error_code_to_result(error_code)?;

        let mut output = [NearestGridpoint::default(); 4];

        for i in 0..4 {
            output[i].lat = output_lats[i];
            output[i].lon = output_lons[i];
            output[i].distance = output_distances[i];
            output[i].index = output_indexes[i];
            output[i].value = output_values[i];
        }

        Ok(output)
    }
}
