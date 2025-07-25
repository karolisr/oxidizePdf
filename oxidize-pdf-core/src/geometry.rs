//! Basic geometric types for PDF

/// A point in 2D space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// X coordinate
    pub x: f64,
    /// Y coordinate  
    pub y: f64,
}

impl Point {
    /// Create a new point
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Origin point (0, 0)
    pub fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// A rectangle defined by two points
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangle {
    /// Lower-left corner
    pub lower_left: Point,
    /// Upper-right corner
    pub upper_right: Point,
}

impl Rectangle {
    /// Create a new rectangle from two points
    pub fn new(lower_left: Point, upper_right: Point) -> Self {
        Self {
            lower_left,
            upper_right,
        }
    }

    /// Create a rectangle from position and size
    pub fn from_position_and_size(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            lower_left: Point::new(x, y),
            upper_right: Point::new(x + width, y + height),
        }
    }

    /// Get the width
    pub fn width(&self) -> f64 {
        self.upper_right.x - self.lower_left.x
    }

    /// Get the height
    pub fn height(&self) -> f64 {
        self.upper_right.y - self.lower_left.y
    }

    /// Get the center point
    pub fn center(&self) -> Point {
        Point::new(
            (self.lower_left.x + self.upper_right.x) / 2.0,
            (self.lower_left.y + self.upper_right.y) / 2.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point() {
        let p = Point::new(10.0, 20.0);
        assert_eq!(p.x, 10.0);
        assert_eq!(p.y, 20.0);

        let origin = Point::origin();
        assert_eq!(origin.x, 0.0);
        assert_eq!(origin.y, 0.0);
    }

    #[test]
    fn test_rectangle() {
        let rect = Rectangle::new(Point::new(10.0, 20.0), Point::new(110.0, 120.0));

        assert_eq!(rect.width(), 100.0);
        assert_eq!(rect.height(), 100.0);

        let center = rect.center();
        assert_eq!(center.x, 60.0);
        assert_eq!(center.y, 70.0);
    }

    #[test]
    fn test_rectangle_from_position_and_size() {
        let rect = Rectangle::from_position_and_size(10.0, 20.0, 50.0, 30.0);
        assert_eq!(rect.lower_left.x, 10.0);
        assert_eq!(rect.lower_left.y, 20.0);
        assert_eq!(rect.upper_right.x, 60.0);
        assert_eq!(rect.upper_right.y, 50.0);
    }
}
