# rust-proj

Rust bindings for [proj.4](https://github.com/OSGeo/proj.4), v5.0.x

# Example
## Inverse Projection from [Stereo70](https://epsg.io/3844) to Geodetic
```rust
extern crate proj;
use proj::Proj;

extern crate geo;
use geo::types::Point;

// reproject coordinates from Stereo70 with custom params into geodetic coordinates (in radians)
let wgs84_name = "+proj=longlat +datum=WGS84 +no_defs";
let wgs84 = Proj::new(wgs84_name).unwrap();
let stereo70 = Proj::new(
    "+proj=sterea +lat_0=46 +lon_0=25 +k=0.99975 +x_0=500000 +y_0=500000 +ellps=krass +towgs84=33.4,-146.6,-76.3,-0.359,-0.053,0.844,-0.84 +units=m +no_defs"
    ).unwrap();
let rp = stereo70.project(&wgs84, Point::new(500000., 500000.));
assert_eq(rp, Point::new(0.436332, 0.802851));
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
