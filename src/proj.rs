use libc::{c_char, c_double};
use std::ffi::CString;
use geo::Point;
use num_traits::Float;
use std::ffi::CStr;
use std::str;

use proj_sys::{proj_context_create, proj_create, proj_destroy, proj_pj_info, proj_trans, PJconsts,
               PJ_COORD, PJ_DIRECTION_PJ_FWD, PJ_DIRECTION_PJ_INV, PJ_XY};

fn _string(raw_ptr: *const c_char) -> String {
    let c_str = unsafe { CStr::from_ptr(raw_ptr) };
    str::from_utf8(c_str.to_bytes()).unwrap().to_string()
}

/// A `proj.4` instance
pub struct Proj {
    c_proj: *mut PJconsts,
}

impl Proj {
    /// Try to instantiate a new `proj.4` projection instance
    /// definition specifies the output projection
    /// if `inverse` is set to `true` when calling `project()`,
    /// output will be a WGS84 lon, lat coord

    // Projection is meant in the sense of `proj.4`'s [definition](http://proj4.org/operations/projections/index.html):
    // "Projections map the spherical 3D space to a flat 2D space."

    // In contrast to proj.4 v4.x, the type of transformation
    // is signalled by the choice of enum used as input to the PJ_COORD union
    // PJ_LP signals projection of geodetic coordinates, with output being PJ_XY
    // and vice versa, or using PJ_XY for both to stay in projected coords
    // TODO: ascertain how this interacts with inverse
    pub fn new(definition: &str) -> Option<Proj> {
        let c_definition = CString::new(definition.as_bytes()).unwrap();
        let ctx = unsafe { proj_context_create() };
        let new_c_proj = unsafe { proj_create(ctx, c_definition.as_ptr()) };
        if new_c_proj.is_null() {
            None
        } else {
            Some(Proj { c_proj: new_c_proj })
        }
    }

    /// Get the current projection's definition from `proj.4`
    pub fn def(&self) -> String {
        let rv = unsafe { proj_pj_info(self.c_proj) };
        _string(rv.definition)
    }
    /// Project the Point coordinates
    pub fn project<T>(&self, point: Point<T>, inverse: bool) -> Point<T>
    where
        T: Float,
    {
        let inv = if inverse {
            PJ_DIRECTION_PJ_INV
        } else {
            PJ_DIRECTION_PJ_FWD
        };
        let c_x: c_double = point.x().to_f64().unwrap();
        let c_y: c_double = point.y().to_f64().unwrap();
        let new_x;
        let new_y;
        // if we define input coords in terms of lambda & phi, using the PJ_LP struct,
        // this signals to proj_trans that we wish to project geodetic coordinates.
        // For conversion (i.e. between projected coordinates) we use
        // PJ_XY {x: , y: }
        let coords = PJ_XY { x: c_x, y: c_y };
        unsafe {
            // PJ_DIRECTION_* determines a forward or inverse transformation
            let trans = proj_trans(self.c_proj, inv, PJ_COORD { xy: coords });
            // output of projected coordinates uses the PJ_COORD:PJ_XY struct
            new_x = trans.xy.x;
            new_y = trans.xy.y;
        }
        Point::new(T::from(new_x).unwrap(), T::from(new_y).unwrap())
    }
}

impl Drop for Proj {
    fn drop(&mut self) {
        unsafe {
            proj_destroy(self.c_proj);
        }
    }
}

#[cfg(test)]
mod test {
    use geo::Point;
    use super::Proj;

    #[test]
    fn test_new_projection() {
        let wgs84 = "+proj=longlat +datum=WGS84 +no_defs";
        let proj = Proj::new(wgs84).unwrap();
        assert_eq!(
            proj.def(),
            "proj=longlat datum=WGS84 no_defs ellps=WGS84 towgs84=0,0,0"
        );
    }

    fn assert_almost_eq(a: f64, b: f64) {
        let f: f64 = a / b;
        assert!(f < 1.00001);
        assert!(f > 0.99999);
    }

    #[test]
    fn test_transform_stereo() {
        let stereo70 = Proj::new(
            "+proj=sterea +lat_0=46 +lon_0=25 +k=0.99975 +x_0=500000 +y_0=500000 +ellps=krass +units=m +no_defs"
            ).unwrap();
        // stereo70 -> WGS84
        let t = stereo70.project(Point::new(500000., 500000.), true);
        assert_almost_eq(t.x(), 0.436332);
        assert_almost_eq(t.y(), 0.802851);
    }
    #[test]
    fn test_transform_wgs84() {
        let stereo70 = Proj::new(
            "+proj=sterea +lat_0=46 +lon_0=25 +k=0.99975 +x_0=500000 +y_0=500000 +ellps=krass +units=m +no_defs"
            ).unwrap();
        // WGS84 -> stereo70
        let t = stereo70.project(Point::new(0.436332, 0.802851), false);
        assert_almost_eq(t.x(), 500000.);
        assert_almost_eq(t.y(), 500000.);
    }
}
