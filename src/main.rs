#[macro_use]
extern crate approx;

use proj_sys::{
    proj_area_destroy, proj_cleanup, proj_context_create, proj_context_destroy, proj_create,
    proj_destroy, proj_errno, proj_errno_reset, proj_trans, PJconsts, PJ_AREA,
    PJ_CONTEXT, PJ_COORD, PJ_DIRECTION_PJ_FWD, PJ_XY,
};
use std::ffi::CString;
use std::str;

pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub struct Proj {
    c_proj: *mut PJconsts,
    ctx: *mut PJ_CONTEXT,
    area: Option<*mut PJ_AREA>,
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

unsafe fn _string(raw_ptr: *const libc::c_char) -> String {
    let c_str = unsafe { std::ffi::CStr::from_ptr(raw_ptr) };
    str::from_utf8(c_str.to_bytes()).expect("Proj gave us invalid string").to_string()
}

/// Look up an error message using the error code
fn error_message(code: libc::c_int) -> String {
    unsafe {
        let ptr = proj_sys::proj_errno_string(code);
        _string(ptr)
    }
}

fn project(definition: &str, point: Point) -> Point {
    let ctx = unsafe { proj_context_create() };
    let c_definition = CString::new(definition).unwrap();
    let new_c_proj = unsafe { proj_create(ctx, c_definition.as_ptr()) };

    unsafe {
        let pj_proj_info = proj_sys::proj_pj_info(new_c_proj);
        println!("id: {}", _string(pj_proj_info.id));
        println!("description: {}", _string(pj_proj_info.description));
        println!("definition: {}", _string(pj_proj_info.definition));
        println!("has_inverse: {}", pj_proj_info.has_inverse == 1);
        println!("accuracy: {}", pj_proj_info.accuracy);
    }

    let proj = Proj {
        c_proj: new_c_proj,
        ctx,
        area: None,
    };

    // let coords = PJ_XY { x: point.x, y: point.y };
    let coords = proj_sys::PJ_XYZT { x: 0.436332, y: 0.802851, z: 0., t: 0. };
    // let coords = unsafe { proj_sys::proj_coord(0.436332, 0.802851, 0., 0.) };

    unsafe {
        println!("point: {:?}", PJ_COORD { xyzt: coords }.v);
        // println!("point: {:?}", coords.v);
    }

    let (new_x, new_y, err) = unsafe {
        proj_errno_reset(proj.c_proj);
        let trans = proj_trans(proj.c_proj, PJ_DIRECTION_PJ_FWD, PJ_COORD { xyzt: coords });
        // let trans = proj_trans(proj.c_proj, PJ_DIRECTION_PJ_FWD, coords);
        (trans.xyzt.x, trans.xyzt.y, proj_errno(proj.c_proj))
    };
    if err != 0 {
        panic!("ERROR: {}",error_message(err));
    }
    Point { x: new_x, y: new_y }
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
