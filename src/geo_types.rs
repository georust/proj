use crate::{Proj, ProjError};
use geo_types;

///```rust
/// # use approx::assert_relative_eq;
/// extern crate proj;
/// use proj::Proj;
/// use geo_types::Coordinate;
///
/// let from = "EPSG:2230";
/// let to = "EPSG:26946";
/// let nad_ft_to_m = Proj::new_known_crs(&from, &to, None).unwrap();
/// let result = nad_ft_to_m
///     .convert(Coordinate { x: 4760096.421921f64, y: 3744293.729449f64 })
///     .unwrap();
/// assert_relative_eq!(result.x, 1450880.29f64, epsilon=1.0e-2);
/// assert_relative_eq!(result.y, 1141263.01f64, epsilon=1.0e-2);
/// ```
impl<T: crate::proj::CoordinateType> crate::Coord<T> for geo_types::Coordinate<T> {
    fn x(&self) -> T {
        self.x
    }
    fn y(&self) -> T {
        self.y
    }
    fn from_xy(x: T, y: T) -> Self {
        Self { x, y }
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
    fn from_xy(x: T, y: T) -> Self {
        Self::new(x, y)
    }
}

impl Proj {
    ///
    /// Convert a [`geo_types::LineString`], using [`Self::convert_array()`] internally
    ///
    pub fn convert_line_string<'a, T>(
        &self,
        line_string: &'a mut geo_types::LineString<T>,
    ) -> Result<&'a mut geo_types::LineString<T>, ProjError>
    where
        T: crate::proj::CoordinateType,
    {
        self.convert_array(&mut line_string.0)?;
        Ok(line_string)
    }

    ///
    /// Convert a [`geo_types::Line`]'s points using [`Self::convert()`]
    ///
    pub fn convert_line<T>(
        &self,
        line: &geo_types::Line<T>,
    ) -> Result<geo_types::Line<T>, ProjError>
    where
        T: crate::proj::CoordinateType,
    {
        Ok(geo_types::Line::new(
            self.convert(line.start)?,
            self.convert(line.end)?,
        ))
    }

    ///
    /// Convert a [`geo_types::Rect`]'s point using [`Self::convert()`]
    ///
    pub fn convert_rect<T>(
        &self,
        rect: &geo_types::Rect<T>,
    ) -> Result<geo_types::Rect<T>, ProjError>
    where
        T: crate::proj::CoordinateType,
    {
        Ok(geo_types::Rect::new(
            self.convert(rect.min())?,
            self.convert(rect.max())?,
        ))
    }

    ///
    /// Convert a [`geo_types::Triangle`]'s point using [`Self::convert()`]
    ///
    pub fn convert_triangle<T>(
        &self,
        triangle: &geo_types::Triangle<T>,
    ) -> Result<geo_types::Triangle<T>, ProjError>
    where
        T: crate::proj::CoordinateType,
    {
        Ok(geo_types::Triangle(
            self.convert(triangle.0)?,
            self.convert(triangle.1)?,
            self.convert(triangle.2)?,
        ))
    }

    ///
    /// Convert a [`geo_types::Polygon`] exterior and interior using [`Self::convert_line_string()`]
    ///
    pub fn convert_polygon<T>(
        &self,
        polygon: geo_types::Polygon<T>,
    ) -> Result<geo_types::Polygon<T>, ProjError>
    where
        T: crate::proj::CoordinateType,
    {
        let (mut exterior, mut interiors) = polygon.into_inner();
        self.convert_line_string(&mut exterior)?;
        for mut interior in interiors.iter_mut() {
            self.convert_line_string(&mut interior)?;
        }
        Ok(geo_types::Polygon::new(exterior, interiors))
    }

    ///
    /// Convert a [`geo_types::MultiPolygon`] using [`Self::convert_polygon()`]
    ///
    pub fn convert_multi_polygon<T>(
        &self,
        multi_polygon: geo_types::MultiPolygon<T>,
    ) -> Result<geo_types::MultiPolygon<T>, ProjError>
    where
        T: crate::proj::CoordinateType,
    {
        Ok(geo_types::MultiPolygon(
            multi_polygon
                .into_iter()
                .map(|p| self.convert_polygon(p))
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }

    ///
    /// Convert [`geo_types::MultiPoint`] using [`Self::convert_array()`]
    ///
    pub fn convert_multi_point<'a, T>(
        &self,
        multi_point: &'a mut geo_types::MultiPoint<T>,
    ) -> Result<&'a mut geo_types::MultiPoint<T>, ProjError>
    where
        T: crate::proj::CoordinateType,
    {
        self.convert_array(&mut multi_point.0)?;
        Ok(multi_point)
    }

    ///
    /// Convert a [`geo_types::MultiLineString`] using [`Self::convert_line_string()`]
    ///
    pub fn convert_multi_line_string<'a, T>(
        &self,
        multi_line_string: &'a mut geo_types::MultiLineString<T>,
    ) -> Result<&'a mut geo_types::MultiLineString<T>, ProjError>
    where
        T: crate::proj::CoordinateType,
    {
        for ls in multi_line_string.into_iter() {
            self.convert_line_string(ls)?;
        }
        Ok(multi_line_string)
    }

    ///
    /// Convert a [`geo_types::Geometry`]
    ///
    pub fn convert_geometry<T>(
        &self,
        geometry: geo_types::Geometry<T>,
    ) -> Result<geo_types::Geometry<T>, ProjError>
    where
        T: crate::proj::CoordinateType,
    {
        match geometry {
            geo_types::Geometry::Point(p) => Ok(self.convert(p)?.into()),
            geo_types::Geometry::Line(mut line) => {
                let _ = self.convert_line(&mut line)?;
                Ok(line.into())
            }
            geo_types::Geometry::LineString(mut ls) => {
                self.convert_line_string(&mut ls)?;
                Ok(ls.into())
            }
            geo_types::Geometry::Polygon(p) => Ok(self.convert_polygon(p)?.into()),
            geo_types::Geometry::MultiPoint(mut multi_point) => {
                self.convert_multi_point(&mut multi_point)?;
                Ok(multi_point.into())
            }
            geo_types::Geometry::MultiLineString(mut multi_line_string) => {
                self.convert_multi_line_string(&mut multi_line_string)?;
                Ok(multi_line_string.into())
            }
            geo_types::Geometry::MultiPolygon(multi_polygon) => {
                Ok(self.convert_multi_polygon(multi_polygon)?.into())
            }
            geo_types::Geometry::GeometryCollection(geometry_collection) => {
                Ok(geo_types::Geometry::GeometryCollection(
                    self.convert_geometry_collection(geometry_collection)?,
                ))
            }
            geo_types::Geometry::Rect(rect) => Ok(self.convert_rect(&rect)?.into()),
            geo_types::Geometry::Triangle(triangle) => Ok(self.convert_triangle(&triangle)?.into()),
        }
    }

    ///
    /// Convert a [`geo_types::GeometryCollection`]
    ///
    pub fn convert_geometry_collection<T>(
        &self,
        geometry_collection: geo_types::GeometryCollection<T>,
    ) -> Result<geo_types::GeometryCollection<T>, ProjError>
    where
        T: crate::proj::CoordinateType,
    {
        Ok(geo_types::GeometryCollection(
            geometry_collection
                .into_iter()
                .map(|g| self.convert_geometry(g))
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use geo_types::{
        line_string, point, polygon, GeometryCollection, LineString, MultiPolygon, Polygon, Rect,
        Triangle,
    };

    fn proj() -> Proj {
        let from = "+proj=lcc +lat_1=49 +lat_2=44 +lat_0=46.5 +lon_0=3 +x_0=700000 +y_0=6600000 +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +units=m +no_defs +type=crs";
        let to = "proj=longlat +datum=WGS84 +no_defs +type=crs";
        Proj::new_known_crs(&from, &to, None).unwrap()
    }

    fn triangle() -> Triangle<f64> {
        Triangle(
            point!(x: 493903.77, y: 6771154.06).into(),
            point!(x: 648773.74, y: 6863725.64).into(),
            point!(x: 740697.71, y: 6745951.11).into(),
        )
    }

    fn rect() -> Rect<f64> {
        Rect::new(
            point!(x: -378305.81, y: 6093283.21),
            point!(x: 1212610.74, y: 7186901.68),
        )
    }

    fn multi_polygon() -> MultiPolygon<f64> {
        MultiPolygon(vec![
            polygon(),
            rect().to_polygon(),
            triangle().to_polygon(),
        ])
    }

    fn line_string() -> LineString<f64> {
        line_string![
            (x: 617466.55, y: 6471839.66),
            (x:  724549.17, y: 6557378.31),
            (x: 806203.24, y: 6497115.20),
        ]
    }

    fn polygon() -> Polygon<f64> {
        polygon!(
            exterior: [
                (x: 459684.42, y:  6902803.84),
                (x: 1008457.04, y: 6842618.16),
                (x: 1066542.47, y: 6313538.74),
                (x: 349688.67, y: 6268474.77),
            ],
            interiors: [
                [
                    (x: 617466.55, y: 6471839.66),
                    (x:  724549.17, y: 6557378.31),
                    (x: 806203.24, y: 6497115.20),
                ],
            ],
        )
    }

    #[test]
    fn test_convert_line_string() {
        let mut input = line_string();
        let output = proj().convert_line_string(&mut input).unwrap();
        insta::assert_debug_snapshot!(output);
    }

    #[test]
    fn test_convert_polygon() {
        let output = proj().convert_polygon(polygon()).unwrap();
        insta::assert_debug_snapshot!(output);
    }

    #[test]
    fn test_convert_rect() {
        let output = proj().convert_rect(&rect()).unwrap();
        insta::assert_debug_snapshot!(output);
    }

    #[test]
    fn test_convert_triangle() {
        let output = proj().convert_triangle(&triangle()).unwrap();
        insta::assert_debug_snapshot!(output);
    }

    #[test]
    fn test_convert_multi_polygon() {
        let output = proj().convert_multi_polygon(multi_polygon()).unwrap();
        insta::assert_debug_snapshot!(output);
    }

    #[test]
    fn test_convert_geometry_collection() {
        let multi_geometry = GeometryCollection(vec![
            triangle().into(),
            line_string().into(),
            rect().into(),
            polygon().into(),
            multi_polygon().into(),
        ]);

        let output = proj().convert_geometry_collection(multi_geometry).unwrap();
        insta::assert_debug_snapshot!(output);
    }
}
