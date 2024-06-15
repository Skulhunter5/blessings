#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    pub const ZERO: Point = Point::new(0, 0);

    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WindowBounds {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl WindowBounds {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}
