#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Rgb(f64, f64, f64),
    Gray(f64),
    Cmyk(f64, f64, f64, f64),
}

impl Color {
    pub fn rgb(r: f64, g: f64, b: f64) -> Self {
        Color::Rgb(
            r.clamp(0.0, 1.0),
            g.clamp(0.0, 1.0),
            b.clamp(0.0, 1.0),
        )
    }
    
    pub fn gray(value: f64) -> Self {
        Color::Gray(value.clamp(0.0, 1.0))
    }
    
    pub fn cmyk(c: f64, m: f64, y: f64, k: f64) -> Self {
        Color::Cmyk(
            c.clamp(0.0, 1.0),
            m.clamp(0.0, 1.0),
            y.clamp(0.0, 1.0),
            k.clamp(0.0, 1.0),
        )
    }
    
    pub fn black() -> Self {
        Color::Gray(0.0)
    }
    
    pub fn white() -> Self {
        Color::Gray(1.0)
    }
    
    pub fn red() -> Self {
        Color::Rgb(1.0, 0.0, 0.0)
    }
    
    pub fn green() -> Self {
        Color::Rgb(0.0, 1.0, 0.0)
    }
    
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