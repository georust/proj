#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
//! Coordinate transformation via bindings to the [PROJ](https://proj.org) v7.1.x API.
//!
//! Two coordinate transformation operations are currently provided: _projection_ (and inverse
//! projection) and _conversion_.
//!
//! _Projection_ is intended for transformations between geodetic and projected coordinates and
//! vice versa (inverse projection), while _conversion_ is intended for transformations between
//! projected coordinate systems. The PROJ [documentation](https://proj.org/operations/index.html)
//! explains the distinction between these operations in more detail.
//!
//! This crate depends on [`libproj v7.1.x`](https://proj.org), accessed via the
//! [`proj-sys`](https://docs.rs/proj-sys) crate. By default, `proj-sys` will try to find a
//! pre-existing installation of libproj on your system. If an appropriate version of libproj
//! cannot be found, the build script will attempt to build libproj from source. You may specify a
//! from-source build with the [`bundled_proj` feature](#feature-flags).
//!
//! Out of the box, any `(x, y)` numeric tuple can be provided as input to proj. You can [conform
//! your own types](#conform-your-own-types) to the [Coord](proj/trait.Coord.html) trait to pass
//! them in directly and avoid intermediate allocations. There is a [`geo-types`
//! feature](#feature-flags), enabled by default, which implements this trait for types in
//! the [`geo-types` crate](https://docs.rs/geo-types).
//!
//! Methods for [conversion](struct.Proj.html#method.convert_array) and
//! [projection](struct.Proj.html#method.project_array) of slices of `Coord`s are also available.
//!
//! # Examples
//!
//! ## Convert from [NAD 83 US Survey Feet](https://epsg.io/2230) to [NAD 83 Meters](https://epsg.io/26946) Using EPSG Codes
//!
//! ```rust
//! # use approx::assert_relative_eq;
//! use proj::Proj;
//!
//! let from = "EPSG:2230";
//! let to = "EPSG:26946";
//! let ft_to_m = Proj::new_known_crs(&from, &to, None).unwrap();
//! let result = ft_to_m
//!     .convert((4760096.421921f64, 3744293.729449f64))
//!     .unwrap();
//! assert_relative_eq!(result.0, 1450880.29, epsilon=1e-2);
//! assert_relative_eq!(result.1, 1141263.01, epsilon=1e-2);
//! ```
//!
//! ## Convert from [NAD 83 US Survey Feet](https://epsg.io/2230) to [NAD 83 Meters](https://epsg.io/26946) Using the `pipeline` Operator
//!
//! Note that as of v5.0.0, PROJ uses the [`pipeline`](https://proj.org/operations/pipeline.html)
//! operator, which allows an arbitrary number of steps in a conversion. The example below works as
//! follows:
//!
//! - define the operation as a `pipeline` operation
//! - define `step` 1 as an `inv`erse transform, yielding geodetic coordinates
//! - define `step` 2 as a forward transform to projected coordinates, yielding metres.
//!
//!
//! ```rust
//! # use approx::assert_relative_eq;
//! use proj::Proj;
//!
//! let ft_to_m = Proj::new("
//!     +proj=pipeline
//!     +step +inv +proj=lcc +lat_1=33.88333333333333
//!     +lat_2=32.78333333333333 +lat_0=32.16666666666666
//!     +lon_0=-116.25 +x_0=2000000.0001016 +y_0=500000.0001016001 +ellps=GRS80
//!     +towgs84=0,0,0,0,0,0,0 +units=us-ft +no_defs
//!     +step +proj=lcc +lat_1=33.88333333333333 +lat_2=32.78333333333333 +lat_0=32.16666666666666
//!     +lon_0=-116.25 +x_0=2000000 +y_0=500000
//!     +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +units=m +no_defs
//! ").unwrap();
//!
//! // The Presidio, approximately
//! let result = ft_to_m.convert((4760096.421921f64, 3744293.729449f64)).unwrap();
//! assert_relative_eq!(result.0, 1450880.29, epsilon=1e-2);
//! assert_relative_eq!(result.1, 1141263.01, epsilon=1e-2);
//! ```
//!
//! # Usage
//!
//! There are two options for creating a transformation:
//!
//! 1. If you don't require additional [grids](#grid-file-download) or other customisation:
//!     - Call `Proj::new` or `Proj::new_known_crs`. This creates a transformation instance ([`Proj`](proj/struct.Proj.html))
//! 2. If you require a grid for the transformation you wish to carry out, or you need to customise
//!    the search path or the grid endpoint:
//!    - Create a new [`ProjBuilder`](proj/struct.ProjBuilder.html) by calling
//!      `ProjBuilder::new()`. It may be modified to enable network downloads, disable the grid,
//!      cache or modify search paths;
//!    - Call [`ProjBuilder.proj()`](proj/struct.ProjBuilder.html#method.proj) or
//!      [`ProjBuilder.proj_known_crs()`](proj/struct.ProjBuilder.html#method.proj_known_crs). This
//!      creates a transformation instance (`Proj`)
//!
//! **Note**:
//!
//! 1. Both `ProjBuilder` and `Proj` implement the [`Info`](proj/trait.Info.html) trait, which can
//!    be used to get information about the current state of the `PROJ` instance;
//! 2. `Proj::new()` and `ProjBuilder::proj()` have the same signature;
//! 3. `Proj::new_known_crs()` and `ProjBuilder::proj_known_crs()` have the same signature.
//!
//! # Requirements
//!
//! By default, the crate requires `libproj` 7.1.x to be present on your system. While it may be
//! backwards-compatible with older PROJ 6 versions, this is neither tested nor supported.
//!
//! # Feature Flags
//!
//! - `geo-types`: include [trait impls for
//!   `geo-types`](proj/trait.Coord.html#impl-Coord%3CT%3E-for-Coordinate%3CT%3E). See
//!   [example](#integration-with-geo-types).
//! - `pkg_config`: enables the use of `pkg-config` when linking against `libproj` —
//!   note that `pkg-config` must be available on your system.
//! - `bundled_proj`: builds `libproj` from source bundled in the `proj-sys` crate.
//!   Note that this feature requires Sqlite3 and `libtiff` to be present on your
//!   system.
//! - `network`: exposes APIs which, when enabled, can fetch grid data from the internet to improve
//!   projection accuracy. See [`enable_network`](struct.ProjBuilder.html#method.enable_network)
//!   for details.
//!
//! ## Network, Cache, and Search Path Functionality
//!
//! ### Grid File Download
//!
//! `proj` supports [network grid download](https://proj.org/usage/network.html) functionality via
//! the [`network` feature](#feature-flags).  Network access is **disabled** by default, and can be
//! activated by passing a `true` `bool` to
//! [`enable_network()`](proj/struct.ProjBuilder.html#method.enable_network).  Network
//! functionality status can be queried with `network_enabled`, and the download endpoint can be
//! queried and set using `get_url_endpoint` and `set_url_endpoint`.
//!
//! #### Grid File Cache
//! Up to 300 mb of downloaded grids are cached to save bandwidth: This cache can be enabled or
//! disabled using [`grid_cache_enable`](proj/struct.ProjBuilder.html#method.grid_cache_enable).
//!
//! ### Search Path Modification
//! The path used to search for resource files can be modified using
//! [`set_search_paths`](proj/struct.ProjBuilder.html#method.set_search_paths)
//!
//! ## Conform your own types
//!
//! If you have your own geometric types, you can conform them to the `Coord` trait and use `proj`
//! without any intermediate allocation.
//!
//! ```rust
//! # use approx::assert_relative_eq;
//! use proj::{Proj, Coord};
//!
//! struct MyPointOfIntereset {
//!     lat: f64,
//!     lon: f64,
//! }
//!
//! impl Coord<f64> for MyPointOfIntereset {
//!     fn x(&self) -> f64 {
//!         self.lon
//!     }
//!     fn y(&self) -> f64 {
//!         self.lat
//!     }
//!     fn from_xy(x: f64, y: f64) -> Self {
//!         Self { lon: x, lat: y }
//!     }
//! }
//!
//! let donut_shop = MyPointOfIntereset { lat: 34.095620, lon: -118.283555 };
//!
//! let from = "EPSG:4326";
//! let to = "EPSG:3309";
//! let proj = Proj::new_known_crs(&from, &to, None).unwrap();
//!
//! let result = proj.convert(donut_shop).unwrap();
//!
//! assert_relative_eq!(result.x(), 158458.67, epsilon=1e-2);
//! assert_relative_eq!(result.y(), -434296.88, epsilon=1e-2);
//! ```
#![cfg_attr(
    feature = "geo-types",
    doc = r##"
## Integration with `geo-types`

If you've enabled the `geo-types` feature, you can skip allocating an intermediate representation,
and pass the [`geo-types`](https://crates.io/crates/geo-types) directly.

```rust
# use approx::assert_relative_eq;
use proj::Proj;
use geo_types::Point;

let my_point = Point::new(4760096.421921f64, 3744293.729449f64);

let from = "EPSG:2230";
let to = "EPSG:26946";
let nad_ft_to_m = Proj::new_known_crs(&from, &to, None).unwrap();

let result = nad_ft_to_m.convert(my_point).unwrap();

assert_relative_eq!(result.x(), 1450880.29, epsilon=1e-2);
assert_relative_eq!(result.y(), 1141263.01, epsilon=1e-2);
```
"##
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "network")]
mod network;

#[cfg_attr(docsrs, feature(doc_cfg))]
#[cfg(feature = "geo-types")]
mod geo_types;

#[cfg(test)]
#[macro_use]
extern crate approx;

mod proj;

pub use crate::proj::Area;
pub use crate::proj::Coord;
pub use crate::proj::Info;
pub use crate::proj::Proj;
pub use crate::proj::ProjBuilder;
pub use crate::proj::ProjError;
pub use crate::proj::Projinfo;
