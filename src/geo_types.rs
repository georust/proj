///```rust
/// # use assert_approx_eq::assert_approx_eq;
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
/// assert_approx_eq!(result.x, 1450880.29f64, 1.0e-2);
/// assert_approx_eq!(result.y, 1141263.01f64, 1.0e-2);
/// ```
impl<T: crate::proj::CoordinateType> crate::Point<T> for geo_types::Coordinate<T> {
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
/// # use assert_approx_eq::assert_approx_eq;
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
/// assert_approx_eq!(result.x(), 1450880.29f64, 1.0e-2);
/// assert_approx_eq!(result.y(), 1141263.01f64, 1.0e-2);
/// ```
impl<T: crate::proj::CoordinateType> crate::Point<T> for geo_types::Point<T> {
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
