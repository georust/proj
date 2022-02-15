use std::{error::Error, fmt};

use crate::{Proj, ProjError};

/// Transform a geometry using PROJ.
pub trait Transform<T> {
    type Output;

    /// Transform a Geometry by mutating it in place.
    ///
    #[cfg_attr(feature = "geo-types", doc = r##"
# Examples

Transform a geometry using a PROJ string definition:

```
use geo_types;
use proj::{Proj, Transform};
 # use approx::assert_relative_eq;

let mut point = geo_types::point!(x: -36.508f32, y: -54.2815f32);
let proj = Proj::new("+proj=axisswap +order=2,1,3,4").expect("invalid proj string");
 point.transform(&proj);

assert_relative_eq!(
    point,
    geo_types::point!(x: -54.2815f32, y: -36.508f32)
);
```
"##)]
    fn transform(&mut self, proj: &Proj) -> Result<(), ProjError>;

    /// Immutable flavor of [`Transform::transform`], which allocates a new geometry.
    ///
    #[cfg_attr(feature = "geo-types", doc = r##"
# Examples

Transform a geometry using a PROJ string definition:

```
use geo_types;
use proj::{Proj, Transform};
# use approx::assert_relative_eq;

let point = geo_types::point!(x: -36.508f32, y: -54.2815f32);
let proj = Proj::new("+proj=axisswap +order=2,1,3,4").expect("invalid proj string");

assert_relative_eq!(
    point.transformed(&proj).unwrap(),
    geo_types::point!(x: -54.2815f32, y: -36.508f32)
);

// original `point` is untouched
assert_relative_eq!(
    point,
    geo_types::point!(x: -36.508f32, y: -54.2815f32)
);
```
"##)]
    fn transformed(&self, proj: &Proj) -> Result<Self::Output, ProjError>;

    /// Transform a geometry from one CRS to another CRS by modifying it in place.
    ///
    #[cfg_attr(feature = "geo-types", doc = r##"
# Examples

```
# use approx::assert_relative_eq;
use proj::Transform;
use geo_types::{point, Point};

let mut point: Point<f32> = point!(x: -36.508f32, y: -54.2815f32);
point.transform_crs_to_crs("EPSG:4326", "EPSG:3857").unwrap();

assert_relative_eq!(point, point!(x: -4064052.0f32, y: -7223650.5f32));
```
"##)]
    fn transform_crs_to_crs(
        &mut self,
        source_crs: &str,
        target_crs: &str,
    ) -> Result<(), TransformError> {
        let proj = Proj::new_known_crs(source_crs, target_crs, None)?;
        Ok(self.transform(&proj)?)
    }

    /// Immutable flavor of [`Transform::transform_crs_to_crs`], which allocates a new geometry.
    ///
    #[cfg_attr(feature = "geo-types", doc = r##"
# Examples

```
# use approx::assert_relative_eq;
use proj::Transform;
use geo_types::{point, Point};

let mut point: Point<f32> = point!(x: -36.508f32, y: -54.2815f32);

assert_relative_eq!(
    point.transformed_crs_to_crs("EPSG:4326", "EPSG:3857").unwrap(),
    point!(x: -4064052.0f32, y: -7223650.5f32)
);
```
"##)]
    fn transformed_crs_to_crs(
        &self,
        source_crs: &str,
        target_crs: &str,
    ) -> Result<Self::Output, TransformError> {
        let proj = Proj::new_known_crs(source_crs, target_crs, None)?;
        Ok(self.transformed(&proj)?)
    }
}

#[derive(Debug)]
pub enum TransformError {
    ProjCreateError(crate::ProjCreateError),
    ProjError(crate::ProjError),
}

impl From<crate::ProjError> for TransformError {
    fn from(e: crate::ProjError) -> Self {
        TransformError::ProjError(e)
    }
}

impl From<crate::ProjCreateError> for TransformError {
    fn from(e: crate::ProjCreateError) -> Self {
        TransformError::ProjCreateError(e)
    }
}

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransformError::ProjCreateError(err) => err.fmt(f),
            TransformError::ProjError(err) => err.fmt(f),
        }
    }
}

impl Error for TransformError {}
