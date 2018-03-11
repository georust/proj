use libc::{c_char, c_double, c_int, c_long, c_void};
use std::ffi::CString;
use geo::Point;
use num_traits::Float;
use std::ffi::CStr;
use std::str;

fn _string(raw_ptr: *const c_char) -> String {
    let c_str = unsafe { CStr::from_ptr(raw_ptr) };
    str::from_utf8(c_str.to_bytes()).unwrap().to_string()
}

/// A `proj.4` instance
pub struct Proj {
    c_proj: *const c_void,
}

#[link(name = "proj")]
extern "C" {
    fn pj_init_plus(definition: *const c_char) -> *const c_void;
    fn pj_free(pj: *const c_void);
    fn pj_get_def(pj: *const c_void) -> *const c_char;
    fn pj_transform(
        srcdefn: *const c_void,
        dstdefn: *const c_void,
        point_count: c_long,
        point_offset: c_int,
        x: *mut c_double,
        y: *mut c_double,
        z: *mut c_double,
    ) -> c_int;
    fn pj_strerrno(code: c_int) -> *const c_char;
}

fn error_message(code: c_int) -> String {
    let rv = unsafe { pj_strerrno(code) };
    _string(rv)
}

impl Proj {
    /// Try to instantiate a new `proj.4` projection instance
    pub fn new(definition: &str) -> Option<Proj> {
        let c_definition = CString::new(definition.as_bytes()).unwrap();
        let c_proj = unsafe { pj_init_plus(c_definition.as_ptr()) };
        if c_proj.is_null() {
            None
        } else {
            Some(Proj { c_proj })
        }
    }

    /// Get the current projection's definition from `proj.4`
    pub fn def(&self) -> String {
        let rv = unsafe { pj_get_def(self.c_proj) };
        _string(rv)
    }
    /// Project a `Point` into `target`'s coordinates
    pub fn project<T>(&self, target: &Proj, point: Point<T>) -> Point<T>
    where
        T: Float,
    {
        let mut c_x: c_double = point.0.x.to_f64().unwrap();
        let mut c_y: c_double = point.0.y.to_f64().unwrap();
        let mut c_z: c_double = 0.;
        unsafe {
            let rv = pj_transform(
                self.c_proj,
                target.c_proj,
                1,
                1,
                &mut c_x,
                &mut c_y,
                &mut c_z,
            );
            if rv != 0 {
                println!("{}", error_message(rv));
            }
            assert!(rv == 0);
        }
        Point::new(T::from(c_x).unwrap(), T::from(c_y).unwrap())
    }
}

impl Drop for Proj {
    fn drop(&mut self) {
        unsafe {
            pj_free(self.c_proj);
        }
    }
}

#[cfg(test)]
mod test {
    use geo::Point;
    use super::Proj;

    #[test]
    fn test_new_projection() {
        let wgs84 = "+proj=longlat +ellps=WGS84 +datum=WGS84 +no_defs";
        let proj = Proj::new(wgs84).unwrap();
        assert_eq!(
            proj.def(),
            " +proj=longlat +ellps=WGS84 +datum=WGS84 +no_defs +towgs84=0,0,0"
        );
    }

    fn assert_almost_eq(a: f64, b: f64) {
        let f: f64 = a / b;
        assert!(f < 1.00001);
        assert!(f > 0.99999);
    }

    #[test]
    fn test_transform() {
        let wgs84_name = "+proj=longlat +datum=WGS84 +no_defs";
        let wgs84 = Proj::new(wgs84_name).unwrap();
        let stereo70 = Proj::new(
            "+proj=sterea +lat_0=46 +lon_0=25 +k=0.99975 +x_0=500000 +y_0=500000 +ellps=krass +units=m +no_defs"
            ).unwrap();

        let rv = stereo70.project(&wgs84, Point::new(500000., 500000.));
        assert_almost_eq(rv.0.x, 0.436332);
        assert_almost_eq(rv.0.y, 0.802851);

        let rv = wgs84.project(&stereo70, Point::new(0.436332, 0.802851));
        assert_almost_eq(rv.0.x, 500000.);
        assert_almost_eq(rv.0.y, 500000.);
    }
}
