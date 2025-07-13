mod color;
mod image;
mod path;

pub use color::Color;
pub use image::{ColorSpace as ImageColorSpace, Image, ImageFormat};
pub use path::{LineCap, LineJoin, PathBuilder};

use crate::error::Result;
use std::fmt::Write;

#[derive(Clone)]
pub struct GraphicsContext {
    operations: String,
    current_color: Color,
    stroke_color: Color,
    line_width: f64,
}

impl Default for GraphicsContext {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphicsContext {
    pub fn new() -> Self {
        Self {
            operations: String::new(),
            current_color: Color::black(),
            stroke_color: Color::black(),
            line_width: 1.0,
        }
    }

    pub fn move_to(&mut self, x: f64, y: f64) -> &mut Self {
        writeln!(&mut self.operations, "{x:.2} {y:.2} m").unwrap();
        self
    }

    pub fn line_to(&mut self, x: f64, y: f64) -> &mut Self {
        writeln!(&mut self.operations, "{x:.2} {y:.2} l").unwrap();
        self
    }

    pub fn curve_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) -> &mut Self {
        writeln!(
            &mut self.operations,
            "{x1:.2} {y1:.2} {x2:.2} {y2:.2} {x3:.2} {y3:.2} c"
        )
        .unwrap();
        self
    }

    pub fn rect(&mut self, x: f64, y: f64, width: f64, height: f64) -> &mut Self {
        writeln!(
            &mut self.operations,
            "{x:.2} {y:.2} {width:.2} {height:.2} re"
        )
        .unwrap();
        self
    }

    pub fn circle(&mut self, cx: f64, cy: f64, radius: f64) -> &mut Self {
        let k = 0.552284749831;
        let r = radius;

        self.move_to(cx + r, cy);
        self.curve_to(cx + r, cy + k * r, cx + k * r, cy + r, cx, cy + r);
        self.curve_to(cx - k * r, cy + r, cx - r, cy + k * r, cx - r, cy);
        self.curve_to(cx - r, cy - k * r, cx - k * r, cy - r, cx, cy - r);
        self.curve_to(cx + k * r, cy - r, cx + r, cy - k * r, cx + r, cy);
        self.close_path()
    }

    pub fn close_path(&mut self) -> &mut Self {
        self.operations.push_str("h\n");
        self
    }

    pub fn stroke(&mut self) -> &mut Self {
        self.apply_stroke_color();
        self.operations.push_str("S\n");
        self
    }

    pub fn fill(&mut self) -> &mut Self {
        self.apply_fill_color();
        self.operations.push_str("f\n");
        self
    }

    pub fn fill_stroke(&mut self) -> &mut Self {
        self.apply_fill_color();
        self.apply_stroke_color();
        self.operations.push_str("B\n");
        self
    }

    pub fn set_stroke_color(&mut self, color: Color) -> &mut Self {
        self.stroke_color = color;
        self
    }

    pub fn set_fill_color(&mut self, color: Color) -> &mut Self {
        self.current_color = color;
        self
    }

    pub fn set_line_width(&mut self, width: f64) -> &mut Self {
        self.line_width = width;
        writeln!(&mut self.operations, "{width:.2} w").unwrap();
        self
    }

    pub fn set_line_cap(&mut self, cap: LineCap) -> &mut Self {
        writeln!(&mut self.operations, "{} J", cap as u8).unwrap();
        self
    }

    pub fn set_line_join(&mut self, join: LineJoin) -> &mut Self {
        writeln!(&mut self.operations, "{} j", join as u8).unwrap();
        self
    }

    pub fn save_state(&mut self) -> &mut Self {
        self.operations.push_str("q\n");
        self
    }

    pub fn restore_state(&mut self) -> &mut Self {
        self.operations.push_str("Q\n");
        self
    }

    pub fn translate(&mut self, tx: f64, ty: f64) -> &mut Self {
        writeln!(&mut self.operations, "1 0 0 1 {tx:.2} {ty:.2} cm").unwrap();
        self
    }

    pub fn scale(&mut self, sx: f64, sy: f64) -> &mut Self {
        writeln!(&mut self.operations, "{sx:.2} 0 0 {sy:.2} 0 0 cm").unwrap();
        self
    }

    pub fn rotate(&mut self, angle: f64) -> &mut Self {
        let cos = angle.cos();
        let sin = angle.sin();
        writeln!(
            &mut self.operations,
            "{:.6} {:.6} {:.6} {:.6} 0 0 cm",
            cos, sin, -sin, cos
        )
        .unwrap();
        self
    }

    pub fn transform(&mut self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> &mut Self {
        writeln!(
            &mut self.operations,
            "{a:.2} {b:.2} {c:.2} {d:.2} {e:.2} {f:.2} cm"
        )
        .unwrap();
        self
    }

    pub fn rectangle(&mut self, x: f64, y: f64, width: f64, height: f64) -> &mut Self {
        self.rect(x, y, width, height)
    }

    pub fn draw_image(
        &mut self,
        image_name: &str,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> &mut Self {
        // Save graphics state
        self.save_state();

        // Set up transformation matrix for image placement
        // PDF coordinate system has origin at bottom-left, so we need to translate and scale
        writeln!(
            &mut self.operations,
            "{width:.2} 0 0 {height:.2} {x:.2} {y:.2} cm"
        )
        .unwrap();

        // Draw the image XObject
        writeln!(&mut self.operations, "/{image_name} Do").unwrap();

        // Restore graphics state
        self.restore_state();

        self
    }

    fn apply_stroke_color(&mut self) {
        match self.stroke_color {
            Color::Rgb(r, g, b) => {
                writeln!(&mut self.operations, "{r:.3} {g:.3} {b:.3} RG").unwrap();
            }
            Color::Gray(g) => {
                writeln!(&mut self.operations, "{g:.3} G").unwrap();
            }
            Color::Cmyk(c, m, y, k) => {
                writeln!(&mut self.operations, "{c:.3} {m:.3} {y:.3} {k:.3} K").unwrap();
            }
        }
    }

    fn apply_fill_color(&mut self) {
        match self.current_color {
            Color::Rgb(r, g, b) => {
                writeln!(&mut self.operations, "{r:.3} {g:.3} {b:.3} rg").unwrap();
            }
            Color::Gray(g) => {
                writeln!(&mut self.operations, "{g:.3} g").unwrap();
            }
            Color::Cmyk(c, m, y, k) => {
                writeln!(&mut self.operations, "{c:.3} {m:.3} {y:.3} {k:.3} k").unwrap();
            }
        }
    }

    pub(crate) fn generate_operations(&self) -> Result<Vec<u8>> {
        Ok(self.operations.as_bytes().to_vec())
    }
}
