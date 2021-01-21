#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
//! # Low-level bindings for PROJ v7.1.x
//!
//! **This is a
//! [`*-sys`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#-sys-packages)
//! crate; you shouldn't use its API directly.** See the
//! [`proj`](https://github.com/georust/proj) crate for general use.
//!
//! A guide to the functions can be found here:
//! <https://proj.org/development/reference/functions.html>.
//!
//! By default, the crate will search for an existing `libproj` (via `PROJ v7.1.x`)
//! installation on your system using
//! [pkg-config](https://www.freedesktop.org/wiki/Software/pkg-config/).
//!
//! If an acceptable installation is not found, proj-sys will attempt to build
//! libproj from source bundled in the crate.
//!
//! ## Features
//!
//! `bundled_proj` - forces building libproj from source even if an acceptable
//! version could be found on your system.  Note that SQLite3 and `libtiff` must be
//! present on your system if you wish to use this feature, and that it builds
//! `libproj` **without** its native network functionality; you will have to
//! implement your own set of callbacks if you wish to make use of them (see the
//! [`proj`](https://crates.io/crates/proj) crate for an example).

#[cfg(not(feature = "nobuild"))]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(feature = "nobuild")]
include!("bindings_docs-rs.rs");

extern "C" {
    #[allow(clashing_extern_declarations)]
    #[link_name = "proj_coord"]
    pub fn proj_coord_2(x: f64, y: f64, z: f64, t: f64) -> PJ_COORD_2;

    #[allow(clashing_extern_declarations)]
    #[link_name = "proj_trans"]
    pub fn proj_trans_2(P: *mut PJ, direction: PJ_DIRECTION, coord: PJ_COORD_2) -> PJ_COORD_2;

    #[allow(clashing_extern_declarations)]
    #[link_name = "proj_trans_array"]
    pub fn proj_trans_array_2(
        P: *mut PJ,
        direction: PJ_DIRECTION,
        n: usize,
        coord: *mut PJ_COORD_2,
    ) -> ::std::os::raw::c_int;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union PJ_COORD_2 {
    pub v: [f64; 4usize],
    pub xyzt: PJ_XYZT,
    pub uvwt: PJ_UVWT,
    pub lpzt: PJ_LPZT,
    pub geod: PJ_GEOD,
    pub opk: PJ_OPK,
    pub enu: PJ_ENU,
    pub xyz: PJ_XYZ,
    pub uvw: PJ_UVW,
    pub lpz: PJ_LPZ,
    pub xy: PJ_XY,
    pub uv: PJ_UV,
    pub lp: PJ_LP,
}
