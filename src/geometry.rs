#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Position { x, y }
    }
}

impl Size {
    pub fn new(width: f64, height: f64) -> Self {
        Size { width, height }
    }
}

impl From<(f64, f64)> for Position {
    fn from((x, y): (f64, f64)) -> Self {
        Self::new(x, y)
    }
}

impl From<(i32, i32)> for Position {
    fn from((x, y): (i32, i32)) -> Self {
        Self::new(x as f64, y as f64)
    }
}

impl Into<(f64, f64)> for Position {
    fn into(self) -> (f64, f64) {
        (self.x, self.y)
    }
}

impl Into<(i32, i32)> for Position {
    fn into(self) -> (i32, i32) {
        (self.x.round() as _, self.x.round() as _)
    }
}

impl From<(f64, f64)> for Size {
    fn from((width, height): (f64, f64)) -> Self {
        Self::new(width, height)
    }
}

impl From<(i32, i32)> for Size {
    fn from((width, height): (i32, i32)) -> Self {
        Self::new(width as f64, height as f64)
    }
}

impl Into<(f64, f64)> for Size {
    fn into(self) -> (f64, f64) {
        (self.width, self.height)
    }
}

impl Into<(i32, i32)> for Size {
    fn into(self) -> (i32, i32) {
        (self.width.round() as _, self.height.round() as _)
    }
}
