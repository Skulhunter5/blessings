use std::io::{self, stdout, Write};

use cell::Cell;
use crossterm::{
    cursor::MoveTo,
    style::{Color, Colors, Print, SetColors},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
    ExecutableCommand, QueueableCommand,
};

mod cell;
mod cursor;
mod util;

pub use cursor::CursorStyle;
use util::Point;
pub use util::WindowBounds;

#[derive(Debug, Clone, Copy)]
pub enum ClearType {
    All,
    CurrentLine,
    UntilNewline,
    Current,
}

#[derive(Debug, Clone)]
pub struct Screen {
    new_screen: Box<[Cell]>,
    cur_screen: Box<[Cell]>,
    width: u16,
    height: u16,
    cursor: Point,
    stored_cursor: Point,
    force_redraw: bool,
    fg_color: Color,
    bg_color: Color,
    new_cursor_style: CursorStyle,
    cur_cursor_style: CursorStyle,
    new_cursor_visibility: bool,
    cur_cursor_visibility: bool,
    windows: Vec<WindowBounds>,
}

impl Screen {
    pub const EMPTY_CHAR: char = Cell::EMPTY_CHAR;
    pub const EMPTY_CELL: Cell = Cell::EMPTY;

    pub fn new() -> io::Result<Self> {
        let (width, height) = terminal::size()?;

        let buffer_size = width as usize * height as usize;
        let new_screen = vec![Screen::EMPTY_CELL; buffer_size].into_boxed_slice();
        let cur_screen = vec![Screen::EMPTY_CELL; buffer_size].into_boxed_slice();

        let cursor = Point::ZERO;
        let stored_cursor = Point::ZERO;

        let fg_color = Color::Reset;
        let bg_color = Color::Reset;

        let new_cursor_style = CursorStyle::DefaultUserShape;
        let cur_cursor_style = CursorStyle::DefaultUserShape;

        let new_cursor_visibility = true;
        let cur_cursor_visibility = true;

        let windows = Vec::new();

        Ok(Self {
            new_screen,
            cur_screen,
            width,
            height,
            cursor,
            stored_cursor,
            force_redraw: false,
            fg_color,
            bg_color,
            new_cursor_style,
            cur_cursor_style,
            new_cursor_visibility,
            cur_cursor_visibility,
            windows,
        })
    }

    pub fn begin(&mut self) -> io::Result<()> {
        let mut stdout = stdout();

        enable_raw_mode()?;
        stdout.queue(EnterAlternateScreen)?;
        stdout.queue(MoveTo(0, 0))?;
        stdout.flush()?;

        Ok(())
    }

    pub fn end(&mut self) -> io::Result<()> {
        let mut stdout = stdout();

        stdout.execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;

        Ok(())
    }

    pub fn get_width(&self) -> u16 {
        if let Some(window) = self.windows.last() {
            window.width
        } else {
            self.width
        }
    }

    pub fn get_height(&self) -> u16 {
        if let Some(window) = self.windows.last() {
            window.height
        } else {
            self.height
        }
    }

    pub fn get_size(&self) -> (u16, u16) {
        if let Some(window) = self.windows.last() {
            (window.width, window.height)
        } else {
            (self.width, self.height)
        }
    }

    fn is_base_window(&self) -> bool {
        self.windows.len() > 0
    }

    fn get_current_window(&self) -> WindowBounds {
        if let Some(window) = self.windows.last() {
            *window
        } else {
            WindowBounds::new(0, 0, self.width, self.height)
        }
    }

    // TODO: maybe switch to errors instead of clamping the window
    pub fn begin_window(&mut self, x: u16, y: u16, width: u16, height: u16) {
        let (outer_x, outer_y, outer_width, outer_height) =
            if let Some(window) = self.windows.last() {
                (window.x, window.y, window.width, window.height)
            } else {
                (0, 0, self.width, self.height)
            };

        let x = (outer_x + x).min(outer_width - 1);
        let y = (outer_y + y).min(outer_height - 1);
        let width = width.min(outer_width - x);
        let height = height.min(outer_height - x);

        self.cursor = Point::ZERO;
        self.stored_cursor = Point::ZERO;

        self.windows.push(WindowBounds::new(x, y, width, height));
    }

    pub fn end_window(&mut self) {
        if let Some(window) = self.windows.pop() {
            self.cursor = Point::new(window.x + self.cursor.x, window.y + self.cursor.y);
            self.stored_cursor = Point::new(
                window.x + self.stored_cursor.x,
                window.y + self.stored_cursor.y,
            );
        }
    }

    pub fn get_cursor(&self) -> (u16, u16) {
        (self.cursor.x, self.cursor.y)
    }

    pub fn get_cursor_style(&self) -> CursorStyle {
        self.new_cursor_style
    }

    pub fn set_cursor_style(&mut self, style: CursorStyle) {
        self.new_cursor_style = style;
    }

    pub fn get_cursor_visibility(&self) -> bool {
        self.new_cursor_visibility
    }

    pub fn set_cursor_visibility(&mut self, visibility: bool) {
        self.new_cursor_visibility = visibility;
    }

    pub fn show_cursor(&mut self) {
        self.set_cursor_visibility(true);
    }

    pub fn hide_cursor(&mut self) {
        self.set_cursor_visibility(false);
    }

    pub fn save_cursor(&mut self) {
        self.stored_cursor = self.cursor;
    }

    pub fn restore_cursor(&mut self) {
        let (width, height) = self.get_size();
        self.cursor = Point::new(
            self.stored_cursor.x.min(width - 1),
            self.stored_cursor.y.min(height - 1),
        );
    }

    pub fn clear(&mut self, ty: ClearType) {
        // TODO: probably start clearing with the current colors

        let window = self.get_current_window();
        match ty {
            ClearType::All => {
                if self.is_base_window() {
                    self.new_screen.fill(Screen::EMPTY_CELL);
                } else {
                    for y in (window.y as usize)..(window.y as usize + window.height as usize) {
                        let index = y * self.width as usize + window.x as usize;
                        self.new_screen[index..(index + window.width as usize)]
                            .fill(Screen::EMPTY_CELL);
                    }
                }
            }
            ClearType::CurrentLine => {
                let start = (window.y + self.cursor.y) as usize * self.width as usize;
                let end = start + window.width as usize;
                self.new_screen[start..end].fill(Screen::EMPTY_CELL);
            }
            ClearType::UntilNewline => {
                let line_start =
                    (window.y + self.cursor.y) as usize * self.width as usize + window.x as usize;
                let start = line_start + self.cursor.x as usize;
                let end = line_start + window.width as usize;
                self.new_screen[start..end].fill(Screen::EMPTY_CELL);
            }
            ClearType::Current => {
                let index = (window.y + self.cursor.y) as usize * self.width as usize
                    + (window.x + self.cursor.x) as usize;
                self.new_screen[index] = Screen::EMPTY_CELL;
            }
        }
    }

    pub fn print<S: AsRef<str>>(&mut self, message: S) {
        let window = self.get_current_window();
        let window_x = window.x as usize;
        let window_y = window.y as usize;
        let window_width = window.width as usize;
        let window_height = window.height as usize;

        let mut x = self.cursor.x as usize;
        let mut y = self.cursor.y as usize;
        let width = self.width as usize;

        message.as_ref().chars().for_each(|c| match c {
            '\n' => {
                x = window_x;
                y += 1;
                if y >= window_x + window_height {
                    y = window_y;
                }
            }
            c => {
                let index = y * width + x;
                self.new_screen[index].fg_color = self.fg_color;
                self.new_screen[index].bg_color = self.bg_color;
                self.new_screen[index].c = c;

                x += 1;
                if x >= window_x + window_width {
                    x = window_x;
                    y += 1;
                    if y >= window_y + window_height {
                        y = window_y;
                    }
                }
            }
        });

        self.cursor = Point::new(x.try_into().unwrap(), y.try_into().unwrap());
    }

    pub fn print_char(&mut self, c: char) {
        let window = self.get_current_window();

        match c {
            '\n' => {
                self.cursor.x = 0;
                self.cursor.y += 1;
                if self.cursor.y >= window.height {
                    self.cursor.y = 0;
                }
            }
            c => {
                // Override cell
                let index = (window.y + self.cursor.y) as usize * self.width as usize
                    + (window.x + self.cursor.x) as usize;
                self.new_screen[index].fg_color = self.fg_color;
                self.new_screen[index].bg_color = self.bg_color;
                self.new_screen[index].c = c;

                self.cursor.x += 1;
                if self.cursor.x >= window.width {
                    self.cursor.x = 0;
                    self.cursor.y += 1;
                    if self.cursor.y >= window.height {
                        self.cursor.y = 0;
                    }
                }
            }
        }
    }

    pub fn print_at<S: AsRef<str>>(&mut self, x: u16, y: u16, message: S) {
        self.move_to(x, y);
        self.print(message);
    }

    pub fn move_to(&mut self, x: u16, y: u16) {
        let window = self.get_current_window();
        self.cursor.x = x.clamp(0, window.width - 1);
        self.cursor.y = y.clamp(0, window.height - 1);
    }

    pub fn clear_colors(&mut self) {
        self.fg_color = Color::Reset;
        self.bg_color = Color::Reset;
    }

    pub fn set_colors(&mut self, foreground_color: Color, background_color: Color) {
        self.fg_color = foreground_color;
        self.bg_color = background_color;
    }

    pub fn set_foreground_color(&mut self, foreground_color: Color) {
        self.fg_color = foreground_color;
    }

    pub fn set_background_color(&mut self, background_color: Color) {
        self.bg_color = background_color;
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        // FIXME: fix all windows after resize or don't allow resize with active windows

        // swap width
        let old_width = self.width;
        let old_height = self.height;
        self.width = width;
        self.height = height;

        // create new buffer
        let mut new_buffer =
            vec![Screen::EMPTY_CELL; width as usize * height as usize].into_boxed_slice();
        // fill new buffer
        let common_width = old_width.min(width) as usize;
        let common_height = old_height.min(height) as usize;
        for i in 0..common_height {
            let old_start = i * old_width as usize;
            let old_end = old_start + common_width;
            let new_start = i * width as usize;
            let new_end = new_start + common_width;
            new_buffer[new_start..new_end].copy_from_slice(&self.new_screen[old_start..old_end]);
        }
        // switch to new buffer
        self.new_screen = new_buffer;
        self.cur_screen = self.new_screen.clone();

        // clamp cursor position to new size
        self.cursor.x = self.cursor.x.min(width - 1);
        self.cursor.y = self.cursor.y.min(height - 1);
        self.stored_cursor.x = self.stored_cursor.x.min(width - 1);
        self.stored_cursor.y = self.stored_cursor.y.min(height - 1);

        self.force_redraw = true;
    }

    pub fn print_whole_screen(&mut self) -> io::Result<()> {
        let mut stdout = stdout();

        let width = self.width as usize;
        let height = self.height as usize;

        let mut fg_color = Color::Reset;
        let mut bg_color = Color::Reset;

        stdout.queue(SetColors(Colors::new(fg_color, bg_color)))?;

        let mut x = 0;
        let mut y = 0;
        let mut i = y * width + x;
        let mut start = i;
        while y < height {
            let new_cell = &self.new_screen[i];

            if new_cell.fg_color != fg_color || new_cell.bg_color != bg_color {
                // print remaining deltas with previous colors
                if start < i {
                    let x = start % width;
                    let y = start / width;
                    stdout.queue(MoveTo(x as u16, y as u16))?;
                    stdout.queue(Print(
                        self.new_screen[start..i]
                            .iter()
                            .map(|cell| cell.c)
                            .collect::<String>(),
                    ))?;
                }

                // change colors
                fg_color = new_cell.fg_color;
                bg_color = new_cell.bg_color;
                stdout.queue(SetColors(Colors::new(fg_color, bg_color)))?;

                start = i;
            }

            i += 1;
            x += 1;
            if x >= width {
                x = 0;
                y += 1;
            }
        }
        if start < i {
            let x = start % width;
            let y = start / width;
            stdout.queue(MoveTo(x as u16, y as u16))?;
            stdout.queue(Print(
                self.new_screen[start..i]
                    .iter()
                    .map(|cell| cell.c)
                    .collect::<String>(),
            ))?;
        }

        Ok(())
    }

    pub fn show(&mut self) -> io::Result<()> {
        let mut stdout = stdout();

        //self.force_redraw = true;
        if self.force_redraw {
            self.print_whole_screen()?;
        } else {
            let width = self.width as usize;
            let height = self.height as usize;

            let mut fg_color = Color::Reset;
            let mut bg_color = Color::Reset;

            stdout.queue(SetColors(Colors::new(fg_color, bg_color)))?;

            let mut x = 0;
            let mut y = 0;
            let mut i = y * width + x;
            let mut start = i;
            while y < height {
                let new_cell = &self.new_screen[i];
                let cur_cell = &self.cur_screen[i];

                if new_cell == cur_cell {
                    if start < i {
                        let x = start % width;
                        let y = start / width;
                        stdout.queue(MoveTo(x as u16, y as u16))?;
                        stdout.queue(Print(
                            self.new_screen[start..i]
                                .iter()
                                .map(|cell| cell.c)
                                .collect::<String>(),
                        ))?;
                    }

                    i += 1;
                    start = i;

                    x += 1;
                    if x >= width {
                        x = 0;
                        y += 1;
                    }
                } else {
                    if new_cell.fg_color != fg_color || new_cell.bg_color != bg_color {
                        // print remaining deltas with previous colors
                        if start < i {
                            let x = start % width;
                            let y = start / width;
                            stdout.queue(MoveTo(x as u16, y as u16))?;
                            stdout.queue(Print(
                                self.new_screen[start..i]
                                    .iter()
                                    .map(|cell| cell.c)
                                    .collect::<String>(),
                            ))?;
                        }

                        // change colors
                        fg_color = new_cell.fg_color;
                        bg_color = new_cell.bg_color;
                        stdout.queue(SetColors(Colors::new(fg_color, bg_color)))?;

                        start = i;
                    }

                    i += 1;
                    x += 1;
                    if x >= width {
                        x = 0;
                        y += 1;
                    }
                }
            }
            if start < i {
                let x = start % width;
                let y = start / width;
                stdout.queue(MoveTo(x as u16, y as u16))?;
                stdout.queue(Print(
                    self.new_screen[start..i]
                        .iter()
                        .map(|cell| cell.c)
                        .collect::<String>(),
                ))?;
            }
        }

        if self.new_cursor_style != self.cur_cursor_style {
            stdout.queue(self.new_cursor_style.to_crossterm_command())?;
            self.cur_cursor_style = self.new_cursor_style;
        }

        if self.new_cursor_visibility != self.cur_cursor_visibility {
            match self.new_cursor_visibility {
                true => stdout.queue(crossterm::cursor::Show),
                false => stdout.queue(crossterm::cursor::Hide),
            }?;
            self.cur_cursor_visibility = self.new_cursor_visibility;
        }

        stdout.queue(MoveTo(self.cursor.x, self.cursor.y))?;
        stdout.flush()?;

        self.force_redraw = false;

        self.cur_screen.copy_from_slice(&self.new_screen);

        Ok(())
    }
}
