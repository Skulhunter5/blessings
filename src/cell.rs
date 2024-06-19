use crossterm::style::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub fg_color: Color,
    pub bg_color: Color,
    pub c: char,
}

impl Cell {
    pub const EMPTY_CHAR: char = ' ';
    pub const EMPTY: Cell = Cell {
        fg_color: Color::Reset,
        bg_color: Color::Reset,
        c: Cell::EMPTY_CHAR,
    };

    pub fn new(fg_color: Color, bg_color: Color, c: char) -> Self {
        Self {
            fg_color,
            bg_color,
            c,
        }
    }
}
