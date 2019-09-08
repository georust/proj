//! `proj` provides bindings to the [PROJ](http://proj.org) v6.2.x API
//!
//! Two coordinate operations are currently provided: projection (and inverse projection)
//! and conversion. Projection is intended for transforming between geodetic and projected coordinates,
//! and vice versa (inverse projection), while conversion is intended for transforming between projected
//! coordinate systems. The PROJ.4 [documentation](http://proj4.org/operations/index.html)
//! explains the distinction between these operations.

#[macro_use]
extern crate failure;
extern crate geo_types;
extern crate libc;
extern crate num_traits;
extern crate proj_sys;

mod proj;

pub use crate::proj::Proj;
pub use crate::proj::Area;
