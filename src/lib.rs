//! `proj` provides bindings to the [proj.4](http://proj4.org), v5.0.x API
//!
//! Two coordinate operations are currently provided: projection (and inverse projection)
//! and conversion. Projection is intended for transforming between geodetic and projected coordinates,
//! and vice versa (inverse projection), while conversion is intended for transforming between projected
//! coordinate systems. The proj.4 [documentation](http://proj4.org/operations/index.html)
//! explains the distinction between these operations.

extern crate num_traits;
extern crate geo;
extern crate libc;
extern crate proj_sys;

mod proj;

pub use proj::Proj;
