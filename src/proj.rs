use libc::{c_char, c_double};
use std::ffi::CString;
use geo::Point;
use num_traits::Float;
use std::ffi::CStr;
use std::str;

use proj_sys::{proj_context_create, proj_create, proj_destroy, proj_pj_info, proj_trans, PJconsts,
               PJ_COORD, PJ_LP, PJ_XY};

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
    /// The definition specifies a **target** projection
    /// Projection is meant in the sense of `proj.4`'s [definition](http://proj4.org/operations/projections/index.html):
    /// "Projections map the spherical 3D space to a flat 2D space."
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
    pub fn project<T>(&self, point: Point<T>) -> Point<T>
    where
        T: Float,
    {
        let c_x: c_double = point.0.x.to_f64().unwrap();
        let c_y: c_double = point.0.y.to_f64().unwrap();
        let new_x;
        let new_y;
        let coords = PJ_LP { lam: c_x, phi: c_y };
        unsafe {
            let trans = proj_trans(self.c_proj, -1, PJ_COORD { lp: coords });
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
    fn test_transform() {
        // EPSG:4326
        // let wgs84 = NProj::new("+proj=longlat +datum=WGS84 +no_defs ").unwrap();
        // EPSG:27700
        let osgb36 = Proj::new("
            +proj=tmerc +lat_0=49 +lon_0=-2 +k=0.9996012717 +x_0=400000 +y_0=-100000 +ellps=airy +towgs84=446.448,-125.157,542.06,0.15,0.247,0.842,-20.489 +units=m +no_defs
            ").unwrap();
        // London, approximately
        let t = osgb36.project(Point::new(0.1290895, 51.5078878));
        assert_almost_eq(t.x(), 529937.885402);
        assert_almost_eq(t.y(), 180432.041828);
    }
}
