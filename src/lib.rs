//! Rust bindings for [PROJ.4](https://github.com/OSGeo/proj.4) v4.9.x

extern crate num_traits;
extern crate geo;
extern crate libc;
extern crate proj_sys;

mod proj;

pub use proj::Proj;
