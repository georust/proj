use libc::c_int;
use libc::{c_char, c_double};
use num_traits::Float;
use proj_sys::{
    proj_area_create, proj_area_destroy, proj_area_set_bbox, proj_cleanup, proj_context_create,
    proj_context_destroy, proj_context_get_url_endpoint, proj_context_is_network_enabled,
    proj_context_set_search_paths, proj_context_set_url_endpoint, proj_create,
    proj_create_crs_to_crs, proj_destroy, proj_errno_string, proj_get_area_of_use,
    proj_grid_cache_set_enable, proj_info, proj_normalize_for_visualization, proj_pj_info,
    proj_trans, proj_trans_array, PJconsts, PJ_AREA, PJ_CONTEXT, PJ_COORD, PJ_DIRECTION_PJ_FWD,
    PJ_DIRECTION_PJ_INV, PJ_INFO, PJ_LP, PJ_XY,
};
use std::fmt::{self, Debug};

#[cfg(feature = "network")]
use proj_sys::proj_context_set_enable_network;

use proj_sys::{proj_errno, proj_errno_reset};

use std::ffi::CStr;
use std::ffi::CString;
use std::mem::MaybeUninit;
use std::path::Path;
use std::str;
use thiserror::Error;

pub trait CoordinateType: Float + Copy + PartialOrd + Debug {}
impl<T: Float + Copy + PartialOrd + Debug> CoordinateType for T {}

/// A point in two dimensional space. The primary unit of input/output for proj.
///
/// By default, any numeric `(x, y)` tuple implements `Coord`, but you can conform your type to
/// `Coord` to pass it directly into proj.
///
/// See the [`geo-types` feature](#feature-flags) for interop with the [`geo-types`
/// crate](https://docs.rs/crate/geo-types)
pub trait Coord<T>
where
    T: CoordinateType,
{
    fn x(&self) -> T;
    fn y(&self) -> T;
    fn from_xy(x: T, y: T) -> Self;
}

impl<T: CoordinateType> Coord<T> for (T, T) {
    fn x(&self) -> T {
        self.0
    }
    fn y(&self) -> T {
        self.1
    }
    fn from_xy(x: T, y: T) -> Self {
        (x, y)
    }
}

/// Errors originating in PROJ which can occur during projection and conversion
#[derive(Error, Debug)]
pub enum ProjError {
    /// A projection error
    #[error("The projection failed with the following error: {0}")]
    Projection(String),
    /// A conversion error
    #[error("The conversion failed with the following error: {0}")]
    Conversion(String),
    /// An error that occurs when a path string originating in PROJ can't be converted to a CString
    #[error("Couldn't create a raw pointer from the string")]
    Creation(#[from] std::ffi::NulError),
    #[error("The projection area of use is unknown")]
    UnknownAreaOfUse,
    /// An error that occurs if a user-supplied path can't be converted into a string slice
    #[error("Couldn't convert path to slice")]
    Path,
    #[error("Couldn't convert bytes from PROJ to UTF-8")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("Couldn't convert number to f64")]
    FloatConversion,
    #[error("Network download functionality could not be enabled")]
    Network,
    #[error("Could not set remote grid download callbacks")]
    RemoteCallbacks,
    #[error("Couldn't build request")]
    #[cfg(feature = "network")]
    BuilderError(#[from] reqwest::Error),
    #[error("Couldn't clone request")]
    RequestCloneError,
    #[error("Could not retrieve content length")]
    ContentLength,
    #[error("Couldn't retrieve header for key {0}")]
    HeaderError(String),
    #[cfg(feature = "network")]
    #[error("Couldn't convert header value to str")]
    HeaderConversion(#[from] reqwest::header::ToStrError),
    #[error("A {0} error occurred for url {1} after {2} retries")]
    DownloadError(String, String, u8),
    #[error("Got a null pointer while attempting to build an error String")]
    NullPointer
}

/// The bounding box of an area of use
///
/// In the case of an area of use crossing the antimeridian (longitude +/- 180 degrees),
/// `west` must be greater than `east`.
#[derive(Copy, Clone, Debug)]
pub struct Area {
    pub north: f64,
    pub south: f64,
    pub east: f64,
    pub west: f64,
}

impl Area {
    /// Create a new Area
    ///
    /// **Note**: In the case of an area of use crossing the antimeridian (longitude +/- 180 degrees),
    /// `west` must be greater than `east`.
    pub fn new(west: f64, south: f64, east: f64, north: f64) -> Self {
        Area {
            west,
            south,
            east,
            north,
        }
    }
}

/// Easily get a String from the external library
pub(crate) unsafe fn _string(raw_ptr: *const c_char) -> Result<String, ProjError> {
    if raw_ptr.is_null() {
        return Err(ProjError::NullPointer);
    }
    let c_str = CStr::from_ptr(raw_ptr);
    Ok(str::from_utf8(c_str.to_bytes())?.to_string())
}

/// Look up an error message using the error code
fn error_message(code: c_int) -> Result<String, ProjError> {
    unsafe {
        let rv = proj_errno_string(code);
        _string(rv)
    }
}

/// Set the bounding box of the area of use
fn area_set_bbox(parea: *mut proj_sys::PJ_AREA, new_area: Option<Area>) {
    // if a bounding box has been passed, modify the proj area object
    if let Some(narea) = new_area {
        unsafe {
            proj_area_set_bbox(parea, narea.west, narea.south, narea.east, narea.north);
        }
    }
}

/// called by Proj::new and ProjBuilder::transform_new_crs
fn transform_string(ctx: *mut PJ_CONTEXT, definition: &str) -> Option<Proj> {
    let c_definition = CString::new(definition).ok()?;
    let new_c_proj = unsafe { proj_create(ctx, c_definition.as_ptr()) };
    if new_c_proj.is_null() {
        None
    } else {
        Some(Proj {
            c_proj: new_c_proj,
            ctx,
            area: None,
        })
    }
}

/// Called by new_known_crs and proj_known_crs
fn transform_epsg(ctx: *mut PJ_CONTEXT, from: &str, to: &str, area: Option<Area>) -> Option<Proj> {
    let from_c = CString::new(from).ok()?;
    let to_c = CString::new(to).ok()?;
    let proj_area = unsafe { proj_area_create() };
    area_set_bbox(proj_area, area);
    let new_c_proj =
        unsafe { proj_create_crs_to_crs(ctx, from_c.as_ptr(), to_c.as_ptr(), proj_area) };
    if new_c_proj.is_null() {
        None
    } else {
        // Normalise input and output order to Lon, Lat / Easting Northing by inserting
        // An axis swap operation if necessary
        let normalised = unsafe {
            let normalised = proj_normalize_for_visualization(ctx, new_c_proj);
            // deallocate stale PJ pointer
            proj_destroy(new_c_proj);
            normalised
        };
        Some(Proj {
            c_proj: normalised,
            ctx,
            area: Some(proj_area),
        })
    }
}

/// Read-only utility methods for providing information about the current PROJ instance
pub trait Info {
    #[doc(hidden)]
    fn ctx(&self) -> *mut PJ_CONTEXT;

    /// Return [Information](https://proj.org/development/reference/datatypes.html#c.PJ_INFO) about the current PROJ context
    /// # Safety
    /// This method contains unsafe code.
    fn info(&self) -> Result<Projinfo, ProjError> {
        unsafe {
            let pinfo: PJ_INFO = proj_info();
            Ok(Projinfo {
                major: pinfo.major,
                minor: pinfo.minor,
                patch: pinfo.patch,
                release: _string(pinfo.release)?,
                version: _string(pinfo.version)?,
                searchpath: _string(pinfo.searchpath)?,
            })
        }
    }

    /// Check whether network access for [resource file download](https://proj.org/resource_files.html#where-are-proj-resource-files-looked-for) is currently enabled or disabled.
    ///
    /// # Safety
    /// This method contains unsafe code.
    fn network_enabled(&self) -> bool {
        let res = unsafe { proj_context_is_network_enabled(self.ctx()) };
        match res {
            1 => true,
            _ => false,
        }
    }

    /// Get the URL endpoint to query for remote grids
    ///
    /// # Safety
    /// This method contains unsafe code.
    fn get_url_endpoint(&self) -> Result<String, ProjError> {
        unsafe { _string(proj_context_get_url_endpoint(self.ctx())) }
    }
}

impl Info for ProjBuilder {
    #[doc(hidden)]
    fn ctx(&self) -> *mut PJ_CONTEXT {
        self.ctx
    }
}

impl ProjBuilder {
    /// Enable or disable network access for [resource file download](https://proj.org/resource_files.html#where-are-proj-resource-files-looked-for).
    ///
    /// # Safety
    /// This method contains unsafe code.
    #[cfg_attr(docsrs, doc(cfg(feature = "network")))]
    #[cfg(feature = "network")]
    pub fn enable_network(&self, enable: bool) -> Result<u8, ProjError> {
        if enable {
            let _ = match crate::network::set_network_callbacks(self.ctx()) {
                1 => Ok(1),
                _ => Err(ProjError::Network),
            }?;
        }
        let enable = if enable { 1 } else { 0 };
        match (enable, unsafe {
            proj_context_set_enable_network(self.ctx(), enable)
        }) {
            // we asked to switch on: switched on
            (1, 1) => Ok(1),
            // we asked to switch off: switched off
            (0, 0) => Ok(0),
            // we asked to switch off, but it's still on
            (0, 1) => Err(ProjError::Network),
            // we asked to switch on, but it's still off
            (1, 0) => Err(ProjError::Network),
            // scrëm
            _ => Err(ProjError::Network),
        }
    }

    /// Add a [resource file search path](https://proj.org/resource_files.html), maintaining existing entries.
    ///
    /// # Safety
    /// This method contains unsafe code.
    pub fn set_search_paths<P: AsRef<Path>>(&self, newpath: P) -> Result<(), ProjError> {
        let existing = self.info()?.searchpath;
        let pathsep = if cfg!(windows) { ";" } else { ":" };
        let mut individual: Vec<&str> = existing.split(pathsep).collect();
        let np = Path::new(newpath.as_ref());
        individual.push(np.to_str().ok_or(ProjError::Path)?);
        let newlength = individual.len() as i32;
        // convert path entries to CString
        let paths_c = individual
            .iter()
            .map(|str| CString::new(*str))
            .collect::<Result<Vec<_>, std::ffi::NulError>>()?;
        // …then to raw pointers
        let paths_p: Vec<_> = paths_c.iter().map(|cstr| cstr.as_ptr()).collect();
        // …then pass the slice of raw pointers as a raw pointer (const char* const*)
        unsafe { proj_context_set_search_paths(self.ctx(), newlength, paths_p.as_ptr()) }
        Ok(())
    }

    /// Enable or disable the local cache of grid chunks
    ///
    /// To avoid repeated network access, a local cache of downloaded chunks of grids is
    /// implemented as SQLite3 database, cache.db, stored in the PROJ user writable directory.
    /// This local caching is **enabled** by default.
    /// The default maximum size of the cache is 300 MB, which is more than half of the total size
    /// of grids available, at time of writing.
    ///
    /// # Safety
    /// This method contains unsafe code.
    pub fn grid_cache_enable(&self, enable: bool) {
        let enable = if enable { 1 } else { 0 };
        let _ = unsafe { proj_grid_cache_set_enable(self.ctx(), enable) };
    }

    /// Set the URL endpoint to query for remote grids
    ///
    /// # Safety
    /// This method contains unsafe code.
    pub fn set_url_endpoint(&self, endpoint: &str) -> Result<(), ProjError> {
        let s = CString::new(endpoint)?;
        unsafe { proj_context_set_url_endpoint(self.ctx(), s.as_ptr()) };
        Ok(())
    }
}

impl Info for Proj {
    #[doc(hidden)]
    fn ctx(&self) -> *mut PJ_CONTEXT {
        self.ctx
    }
}

enum Transformation {
    Projection,
    Conversion,
}

/// [Information](https://proj.org/development/reference/datatypes.html#c.PJ_INFO) about PROJ
#[derive(Clone, Debug)]
pub struct Projinfo {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
    pub release: String,
    pub version: String,
    pub searchpath: String,
}

/// A `PROJ` Context instance, used to create a transformation object.
///
/// Create a transformation object by calling `proj` or `proj_known_crs`.
pub struct ProjBuilder {
    ctx: *mut PJ_CONTEXT,
}

impl ProjBuilder {
    /// Create a new `ProjBuilder`, allowing grid downloads and other customisation.
    pub fn new() -> Self {
        let ctx = unsafe { proj_context_create() };
        ProjBuilder { ctx }
    }

    /// Try to create a coordinate transformation object
    ///
    /// **Note:** for projection operations, `definition` specifies
    /// the **output** projection; input coordinates
    /// are assumed to be geodetic in radians, unless an inverse projection is intended.
    ///
    /// For conversion operations, `definition` defines input, output, and
    /// any intermediate steps that are required. See the `convert` example for more details.
    ///
    /// # Safety
    /// This method contains unsafe code.
    pub fn proj(mut self, definition: &str) -> Option<Proj> {
        let ctx = unsafe { std::mem::replace(&mut self.ctx, proj_context_create()) };
        Some(transform_string(ctx, definition)?)
    }

    /// Try to create a transformation object that is a pipeline between two known coordinate reference systems.
    /// `from` and `to` can be:
    ///
    /// - an `"AUTHORITY:CODE"`, like `"EPSG:25832"`.
    /// - a PROJ string, like `"+proj=longlat +datum=WGS84"`. When using that syntax, the unit is expected to be degrees.
    /// - the name of a CRS as found in the PROJ database, e.g `"WGS84"`, `"NAD27"`, etc.
    /// - more generally, any string accepted by [`new()`](struct.Proj.html#method.new)
    ///
    /// If you wish to alter the particular area of use, you may do so using [`area_set_bbox()`](struct.Proj.html#method.area_set_bbox)
    /// ## A Note on Coordinate Order
    /// The required input **and** output coordinate order is **normalised** to `Longitude, Latitude` / `Easting, Northing`.
    ///
    /// This overrides the expected order of the specified input and / or output CRS if necessary.
    /// See the [PROJ API](https://proj.org/development/reference/functions.html#c.proj_normalize_for_visualization)
    ///
    /// For example: per its definition, EPSG:4326 has an axis order of Latitude, Longitude. Without
    /// normalisation, crate users would have to
    /// [remember](https://proj.org/development/reference/functions.html#c.proj_create_crs_to_crs)
    /// to reverse the coordinates of `Point` or `Coordinate` structs in order for a conversion operation to
    /// return correct results.
    ///
    ///```rust
    /// # use approx::assert_relative_eq;
    /// extern crate proj;
    /// use proj::{Proj, Coord};
    ///
    /// let from = "EPSG:2230";
    /// let to = "EPSG:26946";
    /// let nad_ft_to_m = Proj::new_known_crs(&from, &to, None).unwrap();
    /// let result = nad_ft_to_m
    ///     .convert((4760096.421921f64, 3744293.729449f64))
    ///     .unwrap();
    /// assert_relative_eq!(result.x(), 1450880.29, epsilon = 1.0e-2);
    /// assert_relative_eq!(result.y(), 1141263.01, epsilon = 1.0e-2);
    /// ```
    ///
    /// # Safety
    /// This method contains unsafe code.
    pub fn proj_known_crs(mut self, from: &str, to: &str, area: Option<Area>) -> Option<Proj> {
        let ctx = unsafe { std::mem::replace(&mut self.ctx, proj_context_create()) };
        Some(transform_epsg(ctx, from, to, area)?)
    }
}

impl Default for ProjBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A coordinate transformation object
pub struct Proj {
    c_proj: *mut PJconsts,
    ctx: *mut PJ_CONTEXT,
    area: Option<*mut PJ_AREA>,
}

impl Proj {
    /// Try to create a new transformation object
    ///
    /// **Note:** for projection operations, `definition` specifies
    /// the **output** projection; input coordinates
    /// are assumed to be geodetic in radians, unless an inverse projection is intended.
    ///
    /// For conversion operations, `definition` defines input, output, and
    /// any intermediate steps that are required. See the `convert` example for more details.
    ///
    /// # Safety
    /// This method contains unsafe code.
    // In contrast to proj v4.x, the type of transformation
    // is signalled by the choice of enum used as input to the PJ_COORD union
    // PJ_LP signals projection of geodetic coordinates, with output being PJ_XY
    // and vice versa, or using PJ_XY for conversion operations
    pub fn new(definition: &str) -> Option<Proj> {
        let ctx = unsafe { proj_context_create() };
        Some(transform_string(ctx, definition)?)
    }

    /// Try to create a new transformation object that is a pipeline between two known coordinate reference systems.
    /// `from` and `to` can be:
    ///
    /// - an `"AUTHORITY:CODE"`, like `"EPSG:25832"`.
    /// - a PROJ string, like `"+proj=longlat +datum=WGS84"`. When using that syntax, the unit is expected to be degrees.
    /// - the name of a CRS as found in the PROJ database, e.g `"WGS84"`, `"NAD27"`, etc.
    /// - more generally, any string accepted by [`new()`](struct.Proj.html#method.new)
    ///
    /// If you wish to alter the particular area of use, you may do so using [`area_set_bbox()`](struct.Proj.html#method.area_set_bbox)
    /// ## A Note on Coordinate Order
    /// The required input **and** output coordinate order is **normalised** to `Longitude, Latitude` / `Easting, Northing`.
    ///
    /// This overrides the expected order of the specified input and / or output CRS if necessary.
    /// See the [PROJ API](https://proj.org/development/reference/functions.html#c.proj_normalize_for_visualization)
    ///
    /// For example: per its definition, EPSG:4326 has an axis order of Latitude, Longitude. Without
    /// normalisation, crate users would have to
    /// [remember](https://proj.org/development/reference/functions.html#c.proj_create_crs_to_crs)
    /// to reverse the coordinates of `Point` or `Coordinate` structs in order for a conversion operation to
    /// return correct results.
    ///
    ///```rust
    /// # use approx::assert_relative_eq;
    /// extern crate proj;
    /// use proj::{Proj, Coord};
    ///
    /// let from = "EPSG:2230";
    /// let to = "EPSG:26946";
    /// let nad_ft_to_m = Proj::new_known_crs(&from, &to, None).unwrap();
    /// let result = nad_ft_to_m
    ///     .convert((4760096.421921f64, 3744293.729449f64))
    ///     .unwrap();
    /// assert_relative_eq!(result.x(), 1450880.29, epsilon=1.0e-2);
    /// assert_relative_eq!(result.y(), 1141263.01, epsilon=1.0e-2);
    /// ```
    ///
    /// # Safety
    /// This method contains unsafe code.
    pub fn new_known_crs(from: &str, to: &str, area: Option<Area>) -> Option<Proj> {
        let ctx = unsafe { proj_context_create() };
        Some(transform_epsg(ctx, from, to, area)?)
    }

    /// Set the bounding box of the area of use
    ///
    /// This bounding box will be used to specify the area of use
    /// for the choice of relevant coordinate operations.
    /// In the case of an area of use crossing the antimeridian (longitude +/- 180 degrees),
    /// `west` **must** be greater than `east`.
    ///
    /// # Safety
    /// This method contains unsafe code.
    // calling this on a non-CRS-to-CRS instance of Proj will be harmless, because self.area will be None
    pub fn area_set_bbox(&mut self, new_bbox: Area) {
        if let Some(new_area) = self.area {
            unsafe {
                proj_area_set_bbox(
                    new_area,
                    new_bbox.west,
                    new_bbox.south,
                    new_bbox.east,
                    new_bbox.north,
                );
            }
        }
    }

    /// Returns the area of use of a projection
    ///
    /// When multiple usages are available, the first one will be returned.
    /// The bounding box coordinates are in degrees.
    ///
    /// According to upstream, both the area of use and the projection name
    /// might have not been defined, so they are optional.
    pub fn area_of_use(&self) -> Result<(Option<Area>, Option<String>), ProjError> {
        let mut out_west_lon_degree = MaybeUninit::uninit();
        let mut out_south_lat_degree = MaybeUninit::uninit();
        let mut out_east_lon_degree = MaybeUninit::uninit();
        let mut out_north_lat_degree = MaybeUninit::uninit();
        let mut out_area_name = MaybeUninit::uninit();
        let res = unsafe {
            proj_get_area_of_use(
                self.ctx,
                self.c_proj,
                out_west_lon_degree.as_mut_ptr(),
                out_south_lat_degree.as_mut_ptr(),
                out_east_lon_degree.as_mut_ptr(),
                out_north_lat_degree.as_mut_ptr(),
                out_area_name.as_mut_ptr(),
            )
        };
        if res == 0 {
            Err(ProjError::UnknownAreaOfUse)
        } else {
            let west = unsafe { out_west_lon_degree.assume_init() };
            let south = unsafe { out_south_lat_degree.assume_init() };
            let east = unsafe { out_east_lon_degree.assume_init() };
            let north = unsafe { out_north_lat_degree.assume_init() };
            let name = unsafe {
                let name = out_area_name.assume_init();
                if !name.is_null() {
                    Some(_string(name)?)
                } else {
                    None
                }
            };

            let area = if west != -1000.0 && south != -1000.0 && east != -1000.0 && north != -1000.0
            {
                Some(Area {
                    west,
                    south,
                    east,
                    north,
                })
            } else {
                None
            };
            Ok((area, name))
        }
    }

    fn pj_info(&self) -> PjInfo {
        unsafe {
            let pj_info = proj_pj_info(self.c_proj);
            let id = if pj_info.id.is_null() {
                None
            } else {
                Some(_string(pj_info.id).expect("PROJ built an invalid string"))
            };
            let description = if pj_info.description.is_null() {
                None
            } else {
                Some(_string(pj_info.description).expect("PROJ built an invalid string"))
            };
            let definition = if pj_info.definition.is_null() {
                None
            } else {
                Some(_string(pj_info.definition).expect("PROJ built an invalid string"))
            };
            let has_inverse = pj_info.has_inverse == 1;
            PjInfo {
                id,
                description,
                definition,
                has_inverse,
                accuracy: pj_info.accuracy,
            }
        }
    }

    /// Get the current definition from `PROJ`
    ///
    /// # Safety
    /// This method contains unsafe code.
    pub fn def(&self) -> Result<String, ProjError> {
        Ok(self
            .pj_info()
            .definition
            .expect("proj_pj_info did not provide a definition"))
    }

    /// Project geodetic coordinates (in radians) into the projection specified by `definition`
    ///
    /// **Note:** specifying `inverse` as `true` carries out an inverse projection *to* geodetic coordinates
    /// (in radians) from the projection specified by `definition`.
    ///
    /// # Safety
    /// This method contains unsafe code.
    pub fn project<C, F>(&self, point: C, inverse: bool) -> Result<C, ProjError>
    where
        C: Coord<F>,
        F: CoordinateType,
    {
        let inv = if inverse {
            PJ_DIRECTION_PJ_INV
        } else {
            PJ_DIRECTION_PJ_FWD
        };
        let c_x: c_double = point.x().to_f64().ok_or(ProjError::FloatConversion)?;
        let c_y: c_double = point.y().to_f64().ok_or(ProjError::FloatConversion)?;
        let new_x;
        let new_y;
        let err;
        // Input coords are defined in terms of lambda & phi, using the PJ_LP struct.
        // This signals that we wish to project geodetic coordinates.
        // For conversion (i.e. between projected coordinates) you should use
        // PJ_XY {x: , y: }
        let coords = PJ_LP { lam: c_x, phi: c_y };
        unsafe {
            proj_errno_reset(self.c_proj);
            // PJ_DIRECTION_* determines a forward or inverse projection
            let trans = proj_trans(self.c_proj, inv, PJ_COORD { lp: coords });
            // output of coordinates uses the PJ_XY struct
            new_x = trans.xy.x;
            new_y = trans.xy.y;
            err = proj_errno(self.c_proj);
        }
        if err == 0 {
            Ok(Coord::from_xy(
                F::from(new_x).ok_or(ProjError::FloatConversion)?,
                F::from(new_y).ok_or(ProjError::FloatConversion)?,
            ))
        } else {
            Err(ProjError::Projection(error_message(err)?))
        }
    }

    /// Convert projected coordinates between coordinate reference systems.
    ///
    /// Input and output CRS may be specified in two ways:
    /// 1. Using the PROJ `pipeline` operator. This method makes use of the [`pipeline`](http://proj4.org/operations/pipeline.html)
    /// functionality available since `PROJ` 5.
    /// This has the advantage of being able to chain an arbitrary combination of projection, conversion,
    /// and transformation steps, allowing for extremely complex operations ([`new`](#method.new))
    /// 2. Using EPSG codes or `PROJ` strings to define input and output CRS ([`new_known_crs`](#method.new_known_crs))
    ///
    /// ## A Note on Coordinate Order
    /// Depending on the method used to instantiate the `Proj` object, coordinate input and output order may vary:
    /// - If you have used [`new`](#method.new), it is assumed that you've specified the order using the input string,
    /// or that you are aware of the required input order and expected output order.
    /// - If you have used [`new_known_crs`](#method.new_known_crs), input and output order are **normalised**
    /// to Longitude, Latitude / Easting, Northing.
    ///
    /// The following example converts from NAD83 US Survey Feet (EPSG 2230) to NAD83 Metres (EPSG 26946)
    ///
    /// ```rust
    /// # use approx::assert_relative_eq;
    /// extern crate proj;
    /// use proj::{Proj, Coord};
    ///
    /// let from = "EPSG:2230";
    /// let to = "EPSG:26946";
    /// let ft_to_m = Proj::new_known_crs(&from, &to, None).unwrap();
    /// let result = ft_to_m
    ///     .convert((4760096.421921, 3744293.729449))
    ///     .unwrap();
    /// assert_relative_eq!(result.x() as f64, 1450880.29, epsilon=1e-2);
    /// assert_relative_eq!(result.y() as f64, 1141263.01, epsilon=1e-2);
    /// ```
    ///
    /// # Safety
    /// This method contains unsafe code.
    pub fn convert<C, F>(&self, point: C) -> Result<C, ProjError>
    where
        C: Coord<F>,
        F: CoordinateType,
    {
        let c_x: c_double = point.x().to_f64().ok_or(ProjError::FloatConversion)?;
        let c_y: c_double = point.y().to_f64().ok_or(ProjError::FloatConversion)?;
        let new_x;
        let new_y;
        let err;
        let coords = PJ_XY { x: c_x, y: c_y };
        unsafe {
            proj_errno_reset(self.c_proj);
            let trans = proj_trans(self.c_proj, PJ_DIRECTION_PJ_FWD, PJ_COORD { xy: coords });
            new_x = trans.xy.x;
            new_y = trans.xy.y;
            err = proj_errno(self.c_proj);
        }
        if err == 0 {
            Ok(C::from_xy(
                F::from(new_x).ok_or(ProjError::FloatConversion)?,
                F::from(new_y).ok_or(ProjError::FloatConversion)?,
            ))
        } else {
            Err(ProjError::Conversion(error_message(err)?))
        }
    }

    /// Convert a mutable slice (or anything that can deref into a mutable slice) of `Coord`s
    ///
    /// The following example converts from NAD83 US Survey Feet (EPSG 2230) to NAD83 Metres (EPSG 26946)
    ///
    /// ## A Note on Coordinate Order
    /// Depending on the method used to instantiate the `Proj` object, coordinate input and output order may vary:
    /// - If you have used [`new`](#method.new), it is assumed that you've specified the order using the input string,
    /// or that you are aware of the required input order and expected output order.
    /// - If you have used [`new_known_crs`](#method.new_known_crs), input and output order are **normalised**
    /// to Longitude, Latitude / Easting, Northing.
    ///
    /// ```rust
    /// use proj::{Proj, Coord};
    ///
    /// # use approx::assert_relative_eq;
    /// // Convert from NAD83(NSRS2007) to NAD83(2011)
    /// let from = "EPSG:4759";
    /// let to = "EPSG:4317";
    /// let NAD83_old_to_new = Proj::new_known_crs(&from, &to, None).unwrap();
    /// let mut v = vec![
    ///     (-98.5421515000, 39.2240867222),
    ///     (-98.3166503906, 38.7112325390),
    /// ];
    /// NAD83_old_to_new.convert_array(&mut v);
    /// assert_relative_eq!(v[0].x(), -98.54, epsilon=1e-2);
    /// assert_relative_eq!(v[1].y(), 38.71, epsilon=1e-2);
    /// ```
    ///
    /// # Safety
    /// This method contains unsafe code.
    // TODO: there may be a way of avoiding some allocations, but transmute won't work because
    // PJ_COORD and Coord<T> are different sizes
    pub fn convert_array<'a, C, F>(&self, points: &'a mut [C]) -> Result<&'a mut [C], ProjError>
    where
        C: Coord<F>,
        F: CoordinateType,
    {
        self.array_general(points, Transformation::Conversion, false)
    }

    /// Project an array of geodetic coordinates (in radians) into the projection specified by `definition`
    ///
    /// **Note:** specifying `inverse` as `true` carries out an inverse projection *to* geodetic coordinates
    /// (in radians) from the projection specified by `definition`.
    ///
    /// ```rust
    /// use proj::{Proj, Coord};
    ///
    /// # use approx::assert_relative_eq;
    /// let from = "EPSG:2230";
    /// let to = "EPSG:26946";
    /// let ft_to_m = Proj::new_known_crs(&from, &to, None).unwrap();
    /// let mut v = vec![
    ///     (4760096.421921, 3744293.729449),
    ///     (4760197.421921, 3744394.729449),
    /// ];
    /// ft_to_m.convert_array(&mut v).unwrap();
    /// assert_relative_eq!(v[0].x(), 1450880.29, epsilon=1e-2);
    /// assert_relative_eq!(v[1].y(), 1141293.79, epsilon=1e-2);
    /// ```
    ///
    /// # Safety
    /// This method contains unsafe code.
    // TODO: there may be a way of avoiding some allocations, but transmute won't work because
    // PJ_COORD and Coord<T> are different sizes
    pub fn project_array<'a, C, F>(
        &self,
        points: &'a mut [C],
        inverse: bool,
    ) -> Result<&'a mut [C], ProjError>
    where
        C: Coord<F>,
        F: CoordinateType,
    {
        self.array_general(points, Transformation::Projection, inverse)
    }

    // array conversion and projection logic is almost identical;
    // transform points in input array into PJ_COORD, transform them, error-check, then re-fill
    // input slice with points. Only the actual transformation ops vary slightly.
    fn array_general<'a, C, F>(
        &self,
        points: &'a mut [C],
        op: Transformation,
        inverse: bool,
    ) -> Result<&'a mut [C], ProjError>
    where
        C: Coord<F>,
        F: CoordinateType,
    {
        let err;
        let trans;
        let inv = if inverse {
            PJ_DIRECTION_PJ_INV
        } else {
            PJ_DIRECTION_PJ_FWD
        };
        // we need PJ_COORD to convert
        let mut pj = points
            .iter()
            .map(|point| {
                let c_x: c_double = point.x().to_f64().ok_or(ProjError::FloatConversion)?;
                let c_y: c_double = point.y().to_f64().ok_or(ProjError::FloatConversion)?;
                Ok(PJ_COORD {
                    xy: PJ_XY { x: c_x, y: c_y },
                })
            })
            .collect::<Result<Vec<_>, ProjError>>()?;
        pj.shrink_to_fit();
        // explicitly create the raw pointer to ensure it lives long enough
        let mp = pj.as_mut_ptr();
        // Transformation operations are slightly different
        match op {
            Transformation::Conversion => unsafe {
                proj_errno_reset(self.c_proj);
                trans = proj_trans_array(self.c_proj, PJ_DIRECTION_PJ_FWD, pj.len(), mp);
                err = proj_errno(self.c_proj);
            },
            Transformation::Projection => unsafe {
                proj_errno_reset(self.c_proj);
                trans = proj_trans_array(self.c_proj, inv, pj.len(), mp);
                err = proj_errno(self.c_proj);
            },
        }
        if err == 0 && trans == 0 {
            // re-fill original slice with Coords
            // feels a bit clunky, but we're guaranteed that pj and points have the same length
            unsafe {
                for (i, coord) in pj.iter().enumerate() {
                    points[i] = Coord::from_xy(
                        F::from(coord.xy.x).ok_or(ProjError::FloatConversion)?,
                        F::from(coord.xy.y).ok_or(ProjError::FloatConversion)?,
                    )
                }
            }
            Ok(points)
        } else {
            Err(ProjError::Projection(error_message(err)?))
        }
    }
}

struct PjInfo {
    id: Option<String>,
    description: Option<String>,
    definition: Option<String>,
    has_inverse: bool,
    accuracy: f64,
}

impl fmt::Debug for Proj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pj_info = self.pj_info();
        f.debug_struct("Proj")
            .field("id", &pj_info.id)
            .field("description", &pj_info.description)
            .field("definition", &pj_info.definition)
            .field("has_inverse", &pj_info.has_inverse)
            .field("accuracy", &pj_info.accuracy)
            .finish()
    }
}

impl Drop for Proj {
    fn drop(&mut self) {
        unsafe {
            if let Some(area) = self.area {
                proj_area_destroy(area)
            }
            proj_destroy(self.c_proj);
            proj_context_destroy(self.ctx);
            // NB do NOT call until proj_destroy and proj_context_destroy have both returned:
            // https://proj.org/development/reference/functions.html#c.proj_cleanup
            proj_cleanup()
        }
    }
}

impl Drop for ProjBuilder {
    fn drop(&mut self) {
        unsafe {
            proj_context_destroy(self.ctx);
            proj_cleanup()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug)]
    struct MyPoint {
        x: f64,
        y: f64,
    }

    impl MyPoint {
        fn new(x: f64, y: f64) -> Self {
            MyPoint { x, y }
        }
    }

    impl Coord<f64> for MyPoint {
        fn x(&self) -> f64 {
            self.x
        }

        fn y(&self) -> f64 {
            self.y
        }

        fn from_xy(x: f64, y: f64) -> Self {
            MyPoint { x, y }
        }
    }

    #[cfg(feature = "network")]
    #[test]
    fn test_network_enabled_conversion() {
        // OSGB 1936
        let from = "EPSG:4277";
        // ETRS89
        let to = "EPSG:4258";

        let online_builder = ProjBuilder::new();
        let offline_builder = ProjBuilder::new();

        assert_eq!(online_builder.network_enabled(), false);
        assert_eq!(offline_builder.network_enabled(), false);

        online_builder.enable_network(true).unwrap();
        assert_eq!(online_builder.network_enabled(), true);
        assert_eq!(offline_builder.network_enabled(), false);

        // Disable caching to ensure we're accessing the network.
        // Cache is stored in proj's [user writeable directory](https://proj.org/resource_files.html#user-writable-directory)
        online_builder.grid_cache_enable(false);

        // I expected the following call to trigger a download, but it doesn't!
        let online_proj = online_builder.proj_known_crs(&from, &to, None).unwrap();
        let offline_proj = offline_builder.proj_known_crs(&from, &to, None).unwrap();

        // download begins here:
        // File to download: uk_os_OSTN15_NTv2_OSGBtoETRS.tif
        let online_t = online_proj
            .convert(MyPoint::new(0.001653, 52.267733))
            .unwrap();
        let offline_t = offline_proj
            .convert(MyPoint::new(0.001653, 52.267733))
            .unwrap();

        // Grid download results in a high-quality OSTN15 conversion
        assert_relative_eq!(online_t.x(), 0.000026091248979289044);
        assert_relative_eq!(online_t.y(), 52.26817146070213);

        // Without the grid download, it's a less precise conversion
        assert_relative_eq!(offline_t.x(), -0.00000014658182154077693);
        assert_relative_eq!(offline_t.y(), 52.26815719726976);
    }

    #[test]
    fn test_definition() {
        let wgs84 = "+proj=longlat +datum=WGS84 +no_defs";
        let proj = Proj::new(wgs84).unwrap();
        assert_eq!(
            proj.def().unwrap(),
            "proj=longlat datum=WGS84 no_defs ellps=WGS84 towgs84=0,0,0"
        );
    }

    #[test]
    fn test_debug() {
        let wgs84 = "+proj=longlat +datum=WGS84 +no_defs";
        let proj = Proj::new(wgs84).unwrap();
        let debug_string = format!("{:?}", proj);
        assert_eq!(
            "Proj { id: Some(\"longlat\"), description: Some(\"PROJ-based coordinate operation\"), definition: Some(\"proj=longlat datum=WGS84 no_defs ellps=WGS84 towgs84=0,0,0\"), has_inverse: true, accuracy: -1.0 }",
            debug_string
        );
    }

    #[test]
    #[should_panic]
    // This failure is a bug in libproj
    fn test_searchpath() {
        let tf = ProjBuilder::new();
        tf.set_search_paths(&"/foo").unwrap();
        let ipath = tf.info().unwrap().searchpath;
        let pathsep = if cfg!(windows) { ";" } else { ":" };
        let individual: Vec<&str> = ipath.split(pathsep).collect();
        assert_eq!(&individual.last().unwrap(), &&"/foo")
    }
    #[test]
    fn test_set_endpoint() {
        let from = "EPSG:4326";
        let to = "EPSG:4326+3855";
        let tf = ProjBuilder::new();
        let ep = tf.get_url_endpoint().unwrap();
        assert_eq!(&ep, "https://cdn.proj.org");
        tf.set_url_endpoint("https://github.com/georust").unwrap();
        let proj = tf.proj_known_crs(&from, &to, None).unwrap();
        let ep = proj.get_url_endpoint().unwrap();
        // Has the new endpoint propagated to the Proj instance?
        assert_eq!(&ep, "https://github.com/georust");
    }
    #[test]
    fn test_from_crs() {
        let from = "EPSG:2230";
        let to = "EPSG:26946";
        let proj = Proj::new_known_crs(&from, &to, None).unwrap();
        let t = proj
            .convert(MyPoint::new(4760096.421921, 3744293.729449))
            .unwrap();
        assert_relative_eq!(t.x(), 1450880.2910605003);
        assert_relative_eq!(t.y(), 1141263.0111604529);
    }
    #[test]
    // Carry out a projection from geodetic coordinates
    fn test_projection() {
        let stereo70 = Proj::new(
            "+proj=sterea +lat_0=46 +lon_0=25 +k=0.99975 +x_0=500000 +y_0=500000
            +ellps=krass +towgs84=33.4,-146.6,-76.3,-0.359,-0.053,0.844,-0.84 +units=m +no_defs",
        )
        .unwrap();
        // Geodetic -> Pulkovo 1942(58) / Stereo70 (EPSG 3844)
        let t = stereo70
            .project(MyPoint::new(0.436332, 0.802851), false)
            .unwrap();
        assert_relative_eq!(t.x(), 500119.7035366755, epsilon = 1e-5);
        assert_relative_eq!(t.y(), 500027.77901023754, epsilon = 1e-5);
    }
    #[test]
    // Carry out an inverse projection to geodetic coordinates
    fn test_inverse_projection() {
        let stereo70 = Proj::new(
            "+proj=sterea +lat_0=46 +lon_0=25 +k=0.99975 +x_0=500000 +y_0=500000
            +ellps=krass +towgs84=33.4,-146.6,-76.3,-0.359,-0.053,0.844,-0.84 +units=m +no_defs",
        )
        .unwrap();
        // Pulkovo 1942(58) / Stereo70 (EPSG 3844) -> Geodetic
        let t = stereo70
            .project(MyPoint::new(500119.70352012233, 500027.77896348457), true)
            .unwrap();
        assert_relative_eq!(t.x(), 0.43633200013698786);
        assert_relative_eq!(t.y(), 0.8028510000110507);
    }
    #[test]
    // Carry out an inverse projection to geodetic coordinates
    fn test_london_inverse() {
        let osgb36 = Proj::new(
            "
            +proj=tmerc +lat_0=49 +lon_0=-2 +k=0.9996012717 +x_0=400000 +y_0=-100000 +ellps=airy
            +towgs84=446.448,-125.157,542.06,0.15,0.247,0.842,-20.489 +units=m +no_defs
            ",
        )
        .unwrap();
        // OSGB36 (EPSG 27700) -> Geodetic
        let t = osgb36
            .project(MyPoint::new(548295.39, 182498.46), true)
            .unwrap();
        assert_relative_eq!(t.x(), 0.0023755864830313977);
        assert_relative_eq!(t.y(), 0.89922748952037);
    }
    #[test]
    // Carry out a conversion from NAD83 feet (EPSG 2230) to NAD83 metres (EPSG 26946)
    fn test_conversion() {
        let nad83_m = Proj::new("
            +proj=pipeline
            +step +inv +proj=lcc +lat_1=33.88333333333333
            +lat_2=32.78333333333333 +lat_0=32.16666666666666
            +lon_0=-116.25 +x_0=2000000.0001016 +y_0=500000.0001016001 +ellps=GRS80
            +towgs84=0,0,0,0,0,0,0 +units=us-ft +no_defs
            +step +proj=lcc +lat_1=33.88333333333333 +lat_2=32.78333333333333 +lat_0=32.16666666666666
            +lon_0=-116.25 +x_0=2000000 +y_0=500000
            +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +units=m +no_defs
        ").unwrap();
        // Presidio, San Francisco
        let t = nad83_m
            .convert(MyPoint::new(4760096.421921, 3744293.729449))
            .unwrap();
        assert_relative_eq!(t.x(), 1450880.2910605003);
        assert_relative_eq!(t.y(), 1141263.01116045);
    }
    #[test]
    // Test that instantiation fails wth bad proj string input
    fn test_init_error() {
        assert!(Proj::new("🦀").is_none());
    }
    #[test]
    fn test_conversion_error() {
        // because step 1 isn't an inverse conversion, it's expecting lon lat input
        let nad83_m = Proj::new(
            "+proj=geos +lon_0=0.00 +lat_0=0.00 +a=6378169.00 +b=6356583.80 +h=35785831.0",
        )
        .unwrap();
        let err = nad83_m
            .convert(MyPoint::new(4760096.421921, 3744293.729449))
            .unwrap_err();
        assert_eq!(
            "The conversion failed with the following error: latitude or longitude exceeded limits",
            err.to_string()
        );
    }

    #[test]
    fn test_error_recovery() {
        let nad83_m = Proj::new(
            "+proj=geos +lon_0=0.00 +lat_0=0.00 +a=6378169.00 +b=6356583.80 +h=35785831.0",
        )
        .unwrap();

        // we expect this first conversion to fail (copied from above test case)
        assert!(nad83_m
            .convert(MyPoint::new(4760096.421921, 3744293.729449))
            .is_err());

        // but a subsequent valid conversion should still be successful
        assert!(nad83_m.convert(MyPoint::new(0.0, 0.0)).is_ok());

        // also test with project() function
        assert!(nad83_m
            .project(MyPoint::new(99999.0, 99999.0), false)
            .is_err());
        assert!(nad83_m.project(MyPoint::new(0.0, 0.0), false).is_ok());
    }

    #[test]
    fn test_array_convert() {
        let from = "EPSG:2230";
        let to = "EPSG:26946";
        let ft_to_m = Proj::new_known_crs(&from, &to, None).unwrap();
        let mut v = vec![
            MyPoint::new(4760096.421921, 3744293.729449),
            MyPoint::new(4760197.421921, 3744394.729449),
        ];
        ft_to_m.convert_array(&mut v).unwrap();
        assert_relative_eq!(v[0].x(), 1450880.2910605003f64);
        assert_relative_eq!(v[1].y(), 1141293.7960220198);
    }

    #[test]
    // Ensure that input and output order are normalised to Lon, Lat / Easting Northing
    // Without normalisation this test would fail, as EPSG:4326 expects Lat, Lon input order.
    fn test_input_order() {
        let from = "EPSG:4326";
        let to = "EPSG:2230";
        let to_feet = Proj::new_known_crs(&from, &to, None).unwrap();
        // 👽
        let usa_m = MyPoint::new(-115.797615, 37.2647978);
        let usa_ft = to_feet.convert(usa_m).unwrap();
        assert_eq!(6693625.67217475, usa_ft.x());
        assert_eq!(3497301.5918027186, usa_ft.y());
    }

    #[test]
    fn test_area_of_use() {
        let proj = Proj::new("EPSG:3035").unwrap();
        let (area, name) = proj.area_of_use().unwrap();
        let area = area.unwrap();
        let name = name.unwrap();
        assert_eq!(area.west, -35.58);
        assert_eq!(area.south, 24.6);
        assert_eq!(area.east, 44.83);
        assert_eq!(area.north, 84.17);
        assert!(name.contains("Europe"));
    }
}
