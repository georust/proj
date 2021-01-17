#[macro_use]
extern crate approx;

use libc::{c_char, c_int};
use proj_sys::{
    proj_area_destroy, proj_cleanup, proj_context_create, proj_context_destroy, proj_create,
    proj_destroy, proj_errno, proj_errno_reset, proj_errno_string, proj_trans, PJconsts, PJ_AREA,
    PJ_CONTEXT, PJ_COORD, PJ_DIRECTION_PJ_FWD, PJ_XY,
};
use std::ffi::{CStr, CString};
use std::str;

fn error_message(code: c_int) -> Result<String, String> {
    unsafe {
        let ptr = proj_errno_string(code);
        _string(ptr)
    }
}

unsafe fn _string(raw_ptr: *const c_char) -> Result<String, String> {
    let c_str = CStr::from_ptr(raw_ptr);
    Ok(str::from_utf8(c_str.to_bytes())
        .map_err(|e| e.to_string())?
        .to_string())
}

pub struct Point {
    pub x: f64,
    pub y: f64,
}

fn transform_string(ctx: *mut PJ_CONTEXT, definition: &str) -> Proj {
    let c_definition = CString::new(definition).unwrap();
    let new_c_proj = unsafe { proj_create(ctx, c_definition.as_ptr()) };
    Proj {
        c_proj: new_c_proj,
        ctx,
        area: None,
    }
}

pub struct Proj {
    c_proj: *mut PJconsts,
    ctx: *mut PJ_CONTEXT,
    area: Option<*mut PJ_AREA>,
}

impl Proj {
    pub fn project(&self, point: Point) -> Result<Point, String> {
        let coords = PJ_XY { x: point.x, y: point.y };
        let (new_x, new_y, err) = unsafe {
            proj_errno_reset(self.c_proj);
            let trans = proj_trans(self.c_proj, PJ_DIRECTION_PJ_FWD, PJ_COORD { xy: coords });
            (trans.xy.x, trans.xy.y, proj_errno(self.c_proj))
        };
        if err == 0 {
            Ok(Point { x: new_x, y: new_y })
        } else {
            Err(error_message(err)?)
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

fn project(definition: &str, point: Point) -> Point {
    let ctx = unsafe { proj_context_create() };
    let p = transform_string(ctx, definition);
    p.project(point).unwrap()
}

fn main() {
    let t = project(
        "+proj=sterea +lat_0=46 +lon_0=25 +k=0.99975 +x_0=500000 +y_0=500000
        +ellps=krass +towgs84=33.4,-146.6,-76.3,-0.359,-0.053,0.844,-0.84 +units=m +no_defs",
        Point {
            x: 0.436332,
            y: 0.802851,
        },
    );
    assert_relative_eq!(t.x, 500119.7035366755, epsilon = 1e-5);
    assert_relative_eq!(t.y, 500027.77901023754, epsilon = 1e-5);
}
