# PROJ

High-level Rust bindings for the latest stable version of [PROJ](https://github.com/OSGeo/proj) (7.0.x), compatible with the [Georust](https://crates.io/geo) ecosystem.

# Requirements

Sqlite3 must be present on your system.

By default, this crate depends on a pre-built library, so PROJ v7.0.x must be present on your system. While this crate may be backwards-compatible with older PROJ 6 versions, this is neither tested nor supported.

You can also choose to link against a PROJ included with (and built from source by) the `proj-sys` crate, upon which this crate is built. To do so, enable the `bundled_proj` Cargo feature. Currently this feature only supports Linux.

# Examples

## Convert from [NAD 83 US Survey Feet](https://epsg.io/2230) to [NAD 83 Meters](https://epsg.io/26946) Using EPSG Codes
```rust
use proj::Proj;

extern crate geo_types;
use geo_types::Point;

let from = "EPSG:2230";
let to = "EPSG:26946";
let ft_to_m = Proj::new_known_crs(&from, &to, None).unwrap();
let result = ft_to_m
    .convert(Point::new(4760096.421921, 3744293.729449))
    .unwrap();
assert_almost_eq(result.x(), 1450880.29);
assert_almost_eq(result.y(), 1141263.01);
```

Note that as of v5.0.0, PROJ uses the [`pipeline`](https://proj.org/operations/pipeline.html) operator, which allows an arbitrary number of steps in a conversion. The example below works as follows:

- define the operation as a `pipeline` operation
- define `step` 1 as an `inv`erse transform, yielding geodetic coordinates
- define `step` 2 as a forward transform to projected coordinates, yielding metres.

## Convert from [NAD 83 US Survey Feet](https://epsg.io/2230) to [NAD 83 Meters](https://epsg.io/26946) Using the `pipeline` Operator
```rust
use proj::Proj;

extern crate geo_types;
use geo_types::Point;

let ft_to_m = Proj::new("
    +proj=pipeline
    +step +inv +proj=lcc +lat_1=33.88333333333333
    +lat_2=32.78333333333333 +lat_0=32.16666666666666
    +lon_0=-116.25 +x_0=2000000.0001016 +y_0=500000.0001016001 +ellps=GRS80
    +towgs84=0,0,0,0,0,0,0 +units=us-ft +no_defs
    +step +proj=lcc +lat_1=33.88333333333333 +lat_2=32.78333333333333 +lat_0=32.16666666666666
    +lon_0=-116.25 +x_0=2000000 +y_0=500000
    +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +units=m +no_defs
").unwrap();
// The Presidio, approximately
let result = ft_to_m.convert(Point::new(4760096.421921, 3744293.729449)).unwrap();
assert_eq!(result.x(), 1450880.29);
assert_eq!(result.y(), 1141263.01);
```

## Inverse Projection from [Stereo70](https://epsg.io/3844) to Geodetic
```rust
use proj::Proj;

extern crate geo_types;
use geo_types::Point;

// Carry out an inverse projection from Pulkovo 1942(58) / Stereo70 (EPSG 3844)
// into geodetic lon and lat coordinates (in radians)
let stereo70 = Proj::new("
    +proj=sterea +lat_0=46 +lon_0=25 +k=0.99975 +x_0=500000 +y_0=500000
    +ellps=krass +towgs84=33.4,-146.6,-76.3,-0.359,-0.053,0.844,-0.84
    +units=m +no_defs
    ").unwrap();
let rp = stereo70.project(
    Point::new(500119.70352012233, 500027.77896348457), true
).unwrap();
assert_eq!(rp, Point::new(0.436332, 0.802851));
```

## Bulk Transformations
The `Proj::convert_array()` method is available for bulk conversions. It accepts a mutable slice (or anything that can `Deref` to a mutable slice) of `Point<T: Float>` or `Into<Point<T: Float>` elements.

# License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
