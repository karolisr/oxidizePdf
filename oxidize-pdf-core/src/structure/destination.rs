//! PDF destinations according to ISO 32000-1 Section 12.3.2

use crate::geometry::Rectangle;
use crate::objects::{Array, Object, ObjectId};

/// PDF destination types
#[derive(Debug, Clone, PartialEq)]
pub enum DestinationType {
    /// Display page with coordinates (left, top) at upper-left corner
    XYZ {
        left: Option<f64>,
        top: Option<f64>,
        zoom: Option<f64>,
    },
    /// Fit entire page in window
    Fit,
    /// Fit width of page in window
    FitH { top: Option<f64> },
    /// Fit height of page in window
    FitV { left: Option<f64> },
    /// Fit rectangle in window
    FitR { rect: Rectangle },
    /// Fit page bounding box in window
    FitB,
    /// Fit width of bounding box
    FitBH { top: Option<f64> },
    /// Fit height of bounding box
    FitBV { left: Option<f64> },
}

/// Page destination reference
#[derive(Debug, Clone)]
pub enum PageDestination {
    /// Page number (0-based)
    PageNumber(u32),
    /// Page object reference
    PageRef(ObjectId),
}

/// PDF destination
#[derive(Debug, Clone)]
pub struct Destination {
    /// Target page
    pub page: PageDestination,
    /// Destination type
    pub dest_type: DestinationType,
}

impl Destination {
    /// Create XYZ destination
    pub fn xyz(
        page: PageDestination,
        left: Option<f64>,
        top: Option<f64>,
        zoom: Option<f64>,
    ) -> Self {
        Self {
            page,
            dest_type: DestinationType::XYZ { left, top, zoom },
        }
    }

    /// Create Fit destination
    pub fn fit(page: PageDestination) -> Self {
        Self {
            page,
            dest_type: DestinationType::Fit,
        }
    }

    /// Create FitH destination
    pub fn fit_h(page: PageDestination, top: Option<f64>) -> Self {
        Self {
            page,
            dest_type: DestinationType::FitH { top },
        }
    }

    /// Create FitV destination
    pub fn fit_v(page: PageDestination, left: Option<f64>) -> Self {
        Self {
            page,
            dest_type: DestinationType::FitV { left },
        }
    }

    /// Create FitR destination
    pub fn fit_r(page: PageDestination, rect: Rectangle) -> Self {
        Self {
            page,
            dest_type: DestinationType::FitR { rect },
        }
    }

    /// Convert to PDF array
    pub fn to_array(&self) -> Array {
        let mut arr = Array::new();

        // Add page reference
        match &self.page {
            PageDestination::PageNumber(num) => {
                arr.push(Object::Integer(*num as i64));
            }
            PageDestination::PageRef(id) => {
                arr.push(Object::Reference(*id));
            }
        }

        // Add destination type
        match &self.dest_type {
            DestinationType::XYZ { left, top, zoom } => {
                arr.push(Object::Name("XYZ".to_string()));
                arr.push(left.map(Object::Real).unwrap_or(Object::Null));
                arr.push(top.map(Object::Real).unwrap_or(Object::Null));
                arr.push(zoom.map(Object::Real).unwrap_or(Object::Null));
            }
            DestinationType::Fit => {
                arr.push(Object::Name("Fit".to_string()));
            }
            DestinationType::FitH { top } => {
                arr.push(Object::Name("FitH".to_string()));
                arr.push(top.map(Object::Real).unwrap_or(Object::Null));
            }
            DestinationType::FitV { left } => {
                arr.push(Object::Name("FitV".to_string()));
                arr.push(left.map(Object::Real).unwrap_or(Object::Null));
            }
            DestinationType::FitR { rect } => {
                arr.push(Object::Name("FitR".to_string()));
                arr.push(Object::Real(rect.lower_left.x));
                arr.push(Object::Real(rect.lower_left.y));
                arr.push(Object::Real(rect.upper_right.x));
                arr.push(Object::Real(rect.upper_right.y));
            }
            DestinationType::FitB => {
                arr.push(Object::Name("FitB".to_string()));
            }
            DestinationType::FitBH { top } => {
                arr.push(Object::Name("FitBH".to_string()));
                arr.push(top.map(Object::Real).unwrap_or(Object::Null));
            }
            DestinationType::FitBV { left } => {
                arr.push(Object::Name("FitBV".to_string()));
                arr.push(left.map(Object::Real).unwrap_or(Object::Null));
            }
        }

        arr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn test_xyz_destination() {
        let dest = Destination::xyz(
            PageDestination::PageNumber(0),
            Some(100.0),
            Some(200.0),
            Some(1.5),
        );

        let arr = dest.to_array();
        assert_eq!(arr.len(), 5);
    }

    #[test]
    fn test_fit_destination() {
        let dest = Destination::fit(PageDestination::PageRef(ObjectId::new(10, 0)));
        let arr = dest.to_array();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_fit_r_destination() {
        let rect = Rectangle {
            lower_left: Point { x: 0.0, y: 0.0 },
            upper_right: Point { x: 100.0, y: 200.0 },
        };
        let dest = Destination::fit_r(PageDestination::PageNumber(2), rect);
        let arr = dest.to_array();
        assert_eq!(arr.len(), 6);
    }
}
