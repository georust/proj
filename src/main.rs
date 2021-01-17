#[macro_use]
extern crate approx;

use libc::{c_int, c_char, c_double};
use proj_sys::{proj_cleanup, proj_errno, PJ_COORD, proj_trans, proj_errno_reset, proj_context_destroy, PJ_DIRECTION_PJ_INV, PJ_DIRECTION_PJ_FWD, proj_context_create, PJ_AREA, PJ_CONTEXT, PJconsts, proj_create, proj_destroy, proj_area_destroy, PJ_LP, proj_errno_string};
use std::ffi::{CString, CStr};
use std::str;
use thiserror::Error;

/// Errors originating in PROJ which can occur during projection and conversion
#[derive(Error, Debug)]
pub enum ProjError {
    #[error("Couldn't create a raw pointer from the string")]
    Creation(#[from] std::ffi::NulError),
    #[error("Couldn't convert bytes from PROJ to UTF-8")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("")]
    FloatConversion,
    #[error("")]
    Projection(String),
}

fn error_message(code: c_int) -> Result<String, ProjError> {
    unsafe {
        let ptr = proj_errno_string(code);
        _string(ptr)
    }
}

unsafe fn _string(raw_ptr: *const c_char) -> Result<String, ProjError> {
    let c_str = CStr::from_ptr(raw_ptr);
    Ok(str::from_utf8(c_str.to_bytes())?.to_string())
}

pub struct Point {
    pub x: f64,
    pub y: f64,
}

fn transform_string(ctx: *mut PJ_CONTEXT, definition: &str) -> Option<Proj> {
    let c_definition = CString::new(definition).ok()?;
    let new_c_proj = unsafe { proj_create(ctx, c_definition.as_ptr()) };
    if new_c_proj.is_null() {
        None
    } else {
        Some(Proj {
            c_proj: new_c_proj,
            ctx,
            area: None,
        })
    }
}

pub struct Proj {
    c_proj: *mut PJconsts,
    ctx: *mut PJ_CONTEXT,
    area: Option<*mut PJ_AREA>,
}

impl Proj {
    pub fn new(definition: &str) -> Option<Proj> {
        let ctx = unsafe { proj_context_create() };
        Some(transform_string(ctx, definition)?)
    }

    pub fn project(&self, point: Point, inverse: bool) -> Result<Point, ProjError> {
        let inv = if inverse {
            PJ_DIRECTION_PJ_INV
        } else {
            PJ_DIRECTION_PJ_FWD
        };
        let c_x: c_double = point.x;
        let c_y: c_double = point.y;
        let new_x;
        let new_y;
        let err;
        // Input coords are defined in terms of lambda & phi, using the PJ_LP struct.
        // This signals that we wish to project geodetic coordinates.
        // For conversion (i.e. between projected coordinates) you should use
        // PJ_XY {x: , y: }
        let coords = PJ_LP { lam: c_x, phi: c_y };
        unsafe {
            proj_errno_reset(self.c_proj);
            // PJ_DIRECTION_* determines a forward or inverse projection
            let trans = proj_trans(self.c_proj, inv, PJ_COORD { lp: coords });
            // output of coordinates uses the PJ_XY struct
            new_x = trans.xy.x;
            new_y = trans.xy.y;
            err = proj_errno(self.c_proj);
        }
        if err == 0 {
            Ok(Point { x: new_x, y: new_y })
        } else {
            Err(ProjError::Projection(error_message(err)?))
        }
    }
}

impl Drop for Proj {
    fn drop(&mut self) {
        unsafe {
            if let Some(area) = self.area {
                proj_area_destroy(area)
            }
            proj_destroy(self.c_proj);
            proj_context_destroy(self.ctx);
            // NB do NOT call until proj_destroy and proj_context_destroy have both returned:
            // https://proj.org/development/reference/functions.html#c.proj_cleanup
            proj_cleanup()
        }
    }
}

fn main() {
    let stereo70 = Proj::new(
        "+proj=sterea +lat_0=46 +lon_0=25 +k=0.99975 +x_0=500000 +y_0=500000
        +ellps=krass +towgs84=33.4,-146.6,-76.3,-0.359,-0.053,0.844,-0.84 +units=m +no_defs",
    )
    .unwrap();
    // Geodetic -> Pulkovo 1942(58) / Stereo70 (EPSG 3844)
    let t = stereo70
        .project(
            Point {
                x: 0.436332,
                y: 0.802851,
            },
            false,
        )
        .unwrap();
    assert_relative_eq!(t.x, 500119.7035366755, epsilon = 1e-5);
    assert_relative_eq!(t.y, 500027.77901023754, epsilon = 1e-5);
}
