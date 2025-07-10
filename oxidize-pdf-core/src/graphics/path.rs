#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum LineCap {
    Butt = 0,
    Round = 1,
    Square = 2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum LineJoin {
    Miter = 0,
    Round = 1,
    Bevel = 2,
}

pub struct PathBuilder {
    commands: Vec<PathCommand>,
}

#[derive(Debug, Clone)]
enum PathCommand {
    MoveTo(f64, f64),
    LineTo(f64, f64),
    CurveTo(f64, f64, f64, f64, f64, f64),
    ClosePath,
}

impl PathBuilder {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn move_to(mut self, x: f64, y: f64) -> Self {
        self.commands.push(PathCommand::MoveTo(x, y));
        self
    }

    pub fn line_to(mut self, x: f64, y: f64) -> Self {
        self.commands.push(PathCommand::LineTo(x, y));
        self
    }

    pub fn curve_to(mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) -> Self {
        self.commands
            .push(PathCommand::CurveTo(x1, y1, x2, y2, x3, y3));
        self
    }

    pub fn close(mut self) -> Self {
        self.commands.push(PathCommand::ClosePath);
        self
    }

    pub fn build(self) -> Vec<PathCommand> {
        self.commands
    }
}
