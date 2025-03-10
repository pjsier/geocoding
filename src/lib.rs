//! This crate provides forward– and reverse-geocoding functionality for Rust.
//! Over time, a variety of providers will be added. Each provider may implement one or both
//! of the `Forward` and `Reverse` traits, which provide forward– and reverse-geocoding methods.
//!
//! Note that for the `reverse` method, the return type is simply `String`,
//! As this is the lowest common denominator reverse-geocoding result.
//! Individual providers may implement additional methods, which return more
//! finely-structured and/or extensive data, and enable more specific query tuning.
//! Coordinate data are specified using the [`Point`](struct.Point.html) struct, which has several
//! convenient `From` implementations to allow for easy construction using primitive types.
//!
//! ### A note on Coordinate Order
//! While individual providers may specify coordinates in either `[Longitude, Latitude]` **or**
//! `[Latitude, Longitude`] order,
//! `Geocoding` **always** requires [`Point`](struct.Point.html) data in `[Longitude, Latitude]` (`x, y`) order,
//! and returns data in that order.
//!
static UA_STRING: &'static str = "Rust-Geocoding";

use chrono;
use failure::Error;
pub use geo_types::{Coordinate, Point};
use num_traits::Float;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Client;
use serde::{Deserialize, Deserializer};

// The OpenCage geocoding provider
pub mod opencage;
pub use crate::opencage::Opencage;

// The OpenStreetMap Nominatim geocoding provider
pub mod openstreetmap;
pub use crate::openstreetmap::Openstreetmap;

/// Reverse-geocode a coordinate.
///
/// This trait represents the most simple and minimal implementation
/// available from a given geocoding provider: an address formatted as a String.
///
/// Examples
///
/// ```
/// use geocoding::{Opencage, Point, Reverse};
///
/// let p = Point::new(2.12870, 41.40139);
/// let oc = Opencage::new("dcdbf0d783374909b3debee728c7cc10".to_string());
/// let res = oc.reverse(&p).unwrap();
/// assert_eq!(
///     res,
///     "Carrer de Calatrava, 68, 08017 Barcelona, Spain"
/// );
/// ```
pub trait Reverse<T>
where
    T: Float,
{
    // NOTE TO IMPLEMENTERS: Point coordinates are lon, lat (x, y)
    // You may have to provide these coordinates in reverse order,
    // depending on the provider's requirements (see e.g. OpenCage)
    fn reverse(&self, point: &Point<T>) -> Result<String, Error>;
}

/// Forward-geocode a coordinate.
///
/// This trait represents the most simple and minimal implementation available
/// from a given geocoding provider: It returns a `Vec` of zero or more `Points`.
///
/// Examples
///
/// ```
/// use geocoding::{Coordinate, Forward, Opencage, Point};
///
/// let oc = Opencage::new("dcdbf0d783374909b3debee728c7cc10".to_string());
/// let address = "Schwabing, München";
/// let res: Vec<Point<f64>> = oc.forward(address).unwrap();
/// assert_eq!(
///     res,
///     vec![Point(Coordinate { x: 11.5761796, y: 48.1599218 })]
/// );
/// ```
pub trait Forward<T>
where
    T: Float,
{
    // NOTE TO IMPLEMENTERS: while returned provider point data may not be in
    // lon, lat (x, y) order, Geocoding requires this order in its output Point
    // data. Please pay attention when using returned data to construct Points
    fn forward(&self, address: &str) -> Result<Vec<Point<T>>, Error>;
}

/// Used to specify a bounding box to search within when forward-geocoding
///
/// - `minimum` refers to the **bottom-left** or **south-west** corner of the bounding box
/// - `maximum` refers to the **top-right** or **north-east** corner of the bounding box.
#[derive(Copy, Clone, Debug)]
pub struct InputBounds<T>
where
    T: Float,
{
    pub minimum_lonlat: Point<T>,
    pub maximum_lonlat: Point<T>,
}

impl<T> InputBounds<T>
where
    T: Float,
{
    /// Create a new `InputBounds` struct by passing 2 `Point`s defining:
    /// - minimum (bottom-left) longitude and latitude coordinates
    /// - maximum (top-right) longitude and latitude coordinates
    pub fn new<U>(minimum_lonlat: U, maximum_lonlat: U) -> InputBounds<T>
    where
        U: Into<Point<T>>,
    {
        InputBounds {
            minimum_lonlat: minimum_lonlat.into(),
            maximum_lonlat: maximum_lonlat.into(),
        }
    }
}

/// Convert borrowed input bounds into the correct String representation
impl<T> From<InputBounds<T>> for String
where
    T: Float,
{
    fn from(ip: InputBounds<T>) -> String {
        // Return in lon, lat order
        format!(
            "{},{},{},{}",
            ip.minimum_lonlat.x().to_f64().unwrap().to_string(),
            ip.minimum_lonlat.y().to_f64().unwrap().to_string(),
            ip.maximum_lonlat.x().to_f64().unwrap().to_string(),
            ip.maximum_lonlat.y().to_f64().unwrap().to_string()
        )
    }
}
