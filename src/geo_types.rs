use crate::{Proj, ProjError, Transform};
use geo_types::{coord, Geometry};

///```rust
/// # use approx::assert_relative_eq;
/// extern crate proj;
/// use proj::Proj;
/// use geo_types::coord;
///
/// let from = "EPSG:2230";
/// let to = "EPSG:26946";
/// let nad_ft_to_m = Proj::new_known_crs(&from, &to, None).unwrap();
/// let result = nad_ft_to_m
///     .convert(coord! { x: 4760096.421921f64, y: 3744293.729449f64 })
///     .unwrap();
/// assert_relative_eq!(result.x, 1450880.29f64, epsilon=1.0e-2);
/// assert_relative_eq!(result.y, 1141263.01f64, epsilon=1.0e-2);
/// ```
impl<T: crate::proj::CoordinateType> crate::Coord<T> for geo_types::Coord<T> {
    fn x(&self) -> T {
        self.x
    }
    fn y(&self) -> T {
        self.y
    }
    fn z(&self) -> T {
        T::zero()
    }
    fn from_xyz(x: T, y: T, _z: T) -> Self {
        coord! { x: x, y: y }
    }
}

///```rust
/// # use approx::assert_relative_eq;
/// extern crate proj;
/// use proj::Proj;
/// use geo_types::Point;
///
/// let from = "EPSG:2230";
/// let to = "EPSG:26946";
/// let nad_ft_to_m = Proj::new_known_crs(&from, &to, None).unwrap();
/// let result = nad_ft_to_m
///     .convert(Point::new(4760096.421921f64, 3744293.729449f64))
///     .unwrap();
/// assert_relative_eq!(result.x(), 1450880.29f64, epsilon=1.0e-2);
/// assert_relative_eq!(result.y(), 1141263.01f64, epsilon=1.0e-2);
/// ```
impl<T: crate::proj::CoordinateType> crate::Coord<T> for geo_types::Point<T> {
    fn x(&self) -> T {
        geo_types::Point::x(*self)
    }
    fn y(&self) -> T {
        geo_types::Point::y(*self)
    }
    fn z(&self) -> T {
        T::zero()
    }
    fn from_xyz(x: T, y: T, _z: T) -> Self {
        Self::new(x, y)
    }
}

impl<T> Transform<T> for geo_types::Geometry<T>
where
    T: crate::proj::CoordinateType,
{
    type Output = Self;

    fn transformed(&self, proj: &Proj) -> Result<Self, ProjError> {
        let mut output = self.clone();
        output.transform(proj)?;
        Ok(output)
    }

    fn transform(&mut self, proj: &Proj) -> Result<(), ProjError> {
        match self {
            Geometry::Point(g) => g.transform(proj),
            Geometry::Line(g) => g.transform(proj),
            Geometry::LineString(g) => g.transform(proj),
            Geometry::Polygon(g) => g.transform(proj),
            Geometry::MultiPoint(g) => g.transform(proj),
            Geometry::MultiLineString(g) => g.transform(proj),
            Geometry::MultiPolygon(g) => g.transform(proj),
            Geometry::GeometryCollection(g) => g.transform(proj),
            Geometry::Rect(g) => g.transform(proj),
            Geometry::Triangle(g) => g.transform(proj),
        }
    }
}

impl<T> Transform<T> for geo_types::Coord<T>
where
    T: crate::proj::CoordinateType,
{
    type Output = Self;

    fn transformed(&self, proj: &Proj) -> Result<Self, ProjError> {
        let mut output = *self;
        output.transform(proj)?;
        Ok(output)
    }

    fn transform(&mut self, proj: &Proj) -> Result<(), ProjError> {
        *self = proj.convert(*self)?;
        Ok(())
    }
}

impl<T> Transform<T> for geo_types::Point<T>
where
    T: crate::proj::CoordinateType,
{
    type Output = Self;

    fn transformed(&self, proj: &Proj) -> Result<Self, ProjError> {
        let mut output = *self;
        output.transform(proj)?;
        Ok(output)
    }

    fn transform(&mut self, proj: &Proj) -> Result<(), ProjError> {
        self.0.transform(proj)
    }
}

impl<T> Transform<T> for geo_types::Line<T>
where
    T: crate::proj::CoordinateType,
{
    type Output = Self;

    fn transformed(&self, proj: &Proj) -> Result<Self, ProjError> {
        let mut output = *self;
        output.transform(proj)?;
        Ok(output)
    }

    fn transform(&mut self, proj: &Proj) -> Result<(), ProjError> {
        self.start.transform(proj)?;
        self.end.transform(proj)?;
        Ok(())
    }
}

impl<T> Transform<T> for geo_types::LineString<T>
where
    T: crate::proj::CoordinateType,
{
    type Output = Self;

    fn transformed(&self, proj: &Proj) -> Result<Self, ProjError> {
        let mut output = self.clone();
        output.transform(proj)?;
        Ok(output)
    }

    fn transform(&mut self, proj: &Proj) -> Result<(), ProjError> {
        proj.convert_array(&mut self.0)?;
        Ok(())
    }
}

impl<T> Transform<T> for geo_types::Polygon<T>
where
    T: crate::proj::CoordinateType,
{
    type Output = Self;

    fn transformed(&self, proj: &Proj) -> Result<Self, ProjError> {
        let mut output = self.clone();
        output.transform(proj)?;
        Ok(output)
    }

    fn transform(&mut self, proj: &Proj) -> Result<(), ProjError> {
        let mut exterior_result = Ok(());
        self.exterior_mut(|exterior| {
            exterior_result = exterior.transform(proj);
        });
        exterior_result?;

        let mut interiors_result = Ok(());
        self.interiors_mut(|interiors| {
            interiors_result = interiors
                .iter_mut()
                .try_for_each(|interior| interior.transform(proj))
        });
        interiors_result?;

        Ok(())
    }
}

impl<T> Transform<T> for geo_types::MultiPoint<T>
where
    T: crate::proj::CoordinateType,
{
    type Output = Self;

    fn transformed(&self, proj: &Proj) -> Result<Self, ProjError> {
        let mut output = self.clone();
        output.transform(proj)?;
        Ok(output)
    }

    fn transform(&mut self, proj: &Proj) -> Result<(), ProjError> {
        proj.convert_array(&mut self.0)?;
        Ok(())
    }
}

impl<T> Transform<T> for geo_types::MultiLineString<T>
where
    T: crate::proj::CoordinateType,
{
    type Output = Self;

    fn transformed(&self, proj: &Proj) -> Result<Self, ProjError> {
        let mut output = self.clone();
        output.transform(proj)?;
        Ok(output)
    }

    fn transform(&mut self, proj: &Proj) -> Result<(), ProjError> {
        for line_string in &mut self.0 {
            line_string.transform(proj)?;
        }
        Ok(())
    }
}

impl<T> Transform<T> for geo_types::MultiPolygon<T>
where
    T: crate::proj::CoordinateType,
{
    type Output = Self;

    fn transformed(&self, proj: &Proj) -> Result<Self, ProjError> {
        let mut output = self.clone();
        output.transform(proj)?;
        Ok(output)
    }

    fn transform(&mut self, proj: &Proj) -> Result<(), ProjError> {
        for polygon in &mut self.0 {
            polygon.transform(proj)?;
        }
        Ok(())
    }
}

impl<T> Transform<T> for geo_types::GeometryCollection<T>
where
    T: crate::proj::CoordinateType,
{
    type Output = Self;

    fn transformed(&self, proj: &Proj) -> Result<Self, ProjError> {
        let mut output = self.clone();
        output.transform(proj)?;
        Ok(output)
    }

    fn transform(&mut self, proj: &Proj) -> Result<(), ProjError> {
        for geometry in &mut self.0 {
            geometry.transform(proj)?;
        }
        Ok(())
    }
}

impl<T> Transform<T> for geo_types::Rect<T>
where
    T: crate::proj::CoordinateType,
{
    type Output = Self;

    fn transformed(&self, proj: &Proj) -> Result<Self, ProjError> {
        let mut output = *self;
        output.transform(proj)?;
        Ok(output)
    }

    fn transform(&mut self, proj: &Proj) -> Result<(), ProjError> {
        let a = self.min();
        let b = self.max();
        let new = geo_types::Rect::new(proj.convert(a)?, proj.convert(b)?);
        *self = new;
        Ok(())
    }
}

impl<T> Transform<T> for geo_types::Triangle<T>
where
    T: crate::proj::CoordinateType,
{
    type Output = Self;

    fn transformed(&self, proj: &Proj) -> Result<Self, ProjError> {
        let mut output = *self;
        output.transform(proj)?;
        Ok(output)
    }

    fn transform(&mut self, proj: &Proj) -> Result<(), ProjError> {
        self.0.transform(proj)?;
        self.1.transform(proj)?;
        self.2.transform(proj)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::{point, MultiPoint, Rect};

    #[test]
    fn test_point() {
        let mut subject = point!(x: 4760096.421921f64, y: 3744293.729449f64);
        subject
            .transform_crs_to_crs("EPSG:2230", "EPSG:26946")
            .unwrap();
        let expected = point!(x: 1450880.29f64, y: 1141263.01f64);
        assert_relative_eq!(subject, expected, epsilon = 0.2);
    }

    #[test]
    fn test_rect() {
        let mut subject = {
            let point_a = point!(x: 4760096.421921f64, y: 3744293.729449f64);
            let point_b = point!(x: 4760196.421921f64, y: 3744393.729449f64);
            Rect::new(point_a, point_b)
        };

        subject
            .transform_crs_to_crs("EPSG:2230", "EPSG:26946")
            .unwrap();
        let expected = {
            let point_a = point!(x: 1450880.2910605022, y:  1141263.0111604782);
            let point_b = point!(x: 1450910.771121464, y: 1141293.4912214363);
            Rect::new(point_a, point_b)
        };
        assert_relative_eq!(subject, expected, epsilon = 0.2);
    }

    #[test]
    fn test_multi_point() {
        let mut subject = {
            let point_a = point!(x: 4760096.421921f64, y: 3744293.729449f64);
            let point_b = point!(x: 4760196.421921f64, y: 3744393.729449f64);
            MultiPoint(vec![point_a, point_b])
        };

        subject
            .transform_crs_to_crs("EPSG:2230", "EPSG:26946")
            .unwrap();
        let expected = {
            let point_a = point!(x: 1450880.2910605022, y:  1141263.0111604782);
            let point_b = point!(x: 1450910.771121464, y: 1141293.4912214363);
            MultiPoint(vec![point_a, point_b])
        };
        assert_relative_eq!(subject, expected, epsilon = 0.2);
    }

    #[test]
    fn test_geometry_collection() {
        let mut subject = {
            let multi_point = {
                let point_a = point!(x: 4760096.421921f64, y: 3744293.729449f64);
                let point_b = point!(x: 4760196.421921f64, y: 3744393.729449f64);
                MultiPoint(vec![point_a, point_b])
            };
            let rect = {
                let point_a = point!(x: 4760096.421921f64, y: 3744293.729449f64);
                let point_b = point!(x: 4760196.421921f64, y: 3744393.729449f64);
                Rect::new(point_a, point_b)
            };
            geo_types::GeometryCollection(vec![Geometry::from(multi_point), Geometry::from(rect)])
        };

        subject
            .transform_crs_to_crs("EPSG:2230", "EPSG:26946")
            .unwrap();
        let expected = {
            let multi_point = {
                let point_a = point!(x: 1450880.2910605022, y:  1141263.0111604782);
                let point_b = point!(x: 1450910.771121464, y: 1141293.4912214363);
                MultiPoint(vec![point_a, point_b])
            };
            let rect = {
                let point_a = point!(x: 1450880.2910605022, y:  1141263.0111604782);
                let point_b = point!(x: 1450910.771121464, y: 1141293.4912214363);
                Rect::new(point_a, point_b)
            };
            geo_types::GeometryCollection(vec![Geometry::from(multi_point), Geometry::from(rect)])
        };
        assert_relative_eq!(subject, expected, epsilon = 0.2);
    }
}
