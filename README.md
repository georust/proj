# rust-proj

Rust bindings for [proj.4](https://github.com/OSGeo/proj.4)

# Example
## Reproject from [Stereo70](https://epsg.io/3844) to [WGS84](https://epsg.io/4326)
```rust
extern crate proj;
use proj::Proj;

extern crate geo;
use geo::types::Point;

# reproject coordinates from Stereo70 with custom params into WGS84 lon and lat coordinates
let wgs84_name = "+proj=longlat +datum=WGS84 +no_defs";
let wgs84 = Proj::new(wgs84_name).unwrap();
let stereo70 = Proj::new(
    "+proj=sterea +lat_0=46 +lon_0=25 +k=0.99975 +x_0=500000 +y_0=500000 +ellps=krass +units=m +no_defs"
    ).unwrap();
let rp = stereo70.project(&wgs84, Point::new(500000., 500000.));
# New Point coords are (0.436332, 0.802851)
```
