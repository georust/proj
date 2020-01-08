#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
//! `proj` provides bindings to the [PROJ](https://proj.org) v6.3.x API
//!
//! Two coordinate operations are currently provided: [projection](struct.Proj.html#method.project)
//! (and inverse projection)
//! and [conversion](struct.Proj.html#method.convert).
//! Projection is intended for transformations between geodetic and projected coordinates,
//! and vice versa (inverse projection), while conversion is intended for transformations between projected
//! coordinate systems. The PROJ [documentation](https://proj.org/operations/index.html)
//! explains the distinction between these operations.
//!
//! Anything that can be converted into a [`geo-types`](https://docs.rs/geo-types) `Point` via the `Into`
//! trait can be used as input for the conversion or transformation functions, and conversion of a slice of
//! `Point`s [is also supported](struct.Proj.html#method.convert_array).
//!
//! # Example
//!
//! ```
//! use assert_approx_eq::assert_approx_eq;
//! extern crate proj;
//! use proj::Proj;
//!
//! extern crate geo_types;
//! use geo_types::Point;
//!
//! let from = "EPSG:2230";
//! let to = "EPSG:26946";
//! let nad_ft_to_m = Proj::new_known_crs(&from, &to, None).unwrap();
//! let result = nad_ft_to_m
//!     .convert(Point::new(4760096.421921f64, 3744293.729449f64))
//!     .unwrap();
//! assert_approx_eq!(result.x(), 1450880.29f64, 1.0e-2);
//! assert_approx_eq!(result.y(), 1141263.01f64, 1.0e-2);
//!```

mod proj;

pub use crate::proj::Area;
pub use crate::proj::Proj;
