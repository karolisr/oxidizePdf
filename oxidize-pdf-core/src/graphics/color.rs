/// Represents a color in PDF documents.
/// 
/// Supports RGB, Grayscale, and CMYK color spaces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    /// RGB color (red, green, blue) with values from 0.0 to 1.0
    Rgb(f64, f64, f64),
    /// Grayscale color with value from 0.0 (black) to 1.0 (white)
    Gray(f64),
    /// CMYK color (cyan, magenta, yellow, key/black) with values from 0.0 to 1.0
    Cmyk(f64, f64, f64, f64),
}

impl Color {
    /// Creates an RGB color with values clamped to 0.0-1.0.
    pub fn rgb(r: f64, g: f64, b: f64) -> Self {
        Color::Rgb(
            r.clamp(0.0, 1.0),
            g.clamp(0.0, 1.0),
            b.clamp(0.0, 1.0),
        )
    }
    
    /// Creates a grayscale color with value clamped to 0.0-1.0.
    pub fn gray(value: f64) -> Self {
        Color::Gray(value.clamp(0.0, 1.0))
    }
    
    /// Creates a CMYK color with values clamped to 0.0-1.0.
    pub fn cmyk(c: f64, m: f64, y: f64, k: f64) -> Self {
        Color::Cmyk(
            c.clamp(0.0, 1.0),
            m.clamp(0.0, 1.0),
            y.clamp(0.0, 1.0),
            k.clamp(0.0, 1.0),
        )
    }
    
    /// Black color (gray 0.0).
    pub fn black() -> Self {
        Color::Gray(0.0)
    }
    
    /// White color (gray 1.0).
    pub fn white() -> Self {
        Color::Gray(1.0)
    }
    
    /// Red color (RGB 1,0,0).
    pub fn red() -> Self {
        Color::Rgb(1.0, 0.0, 0.0)
    }
    
    /// Green color (RGB 0,1,0).
    pub fn green() -> Self {
        Color::Rgb(0.0, 1.0, 0.0)
    }
    
    /// Blue color (RGB 0,0,1).
    pub fn blue() -> Self {
        Color::Rgb(0.0, 0.0, 1.0)
    }
    
    pub fn yellow() -> Self {
        Color::Rgb(1.0, 1.0, 0.0)
    }
    
    pub fn cyan() -> Self {
        Color::Rgb(0.0, 1.0, 1.0)
    }
    
    pub fn magenta() -> Self {
        Color::Rgb(1.0, 0.0, 1.0)
    }
}