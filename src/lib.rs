use std::io::{self, stdout, Write};

use cell::Cell;
use crossterm::{cursor::MoveTo, style::{Color, Colors, Print, SetColors}, terminal::{self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand, QueueableCommand};

mod cell;

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
    cursor_x: u16,
    cursor_y: u16,
    stored_x: u16,
    stored_y: u16,
    force_redraw: bool,
    // TODO: remove redraws counter
    redraws: usize,
    fg_color: Color,
    bg_color: Color,
}

impl Screen {
    pub const EMPTY_CHAR: char = Cell::EMPTY_CHAR;
    pub const EMPTY_CELL: Cell = Cell::EMPTY;

    pub fn new() -> io::Result<Self> {
        let (width, height) = terminal::size()?;

        let buffer_size = width as usize * height as usize;
        let new_screen = vec![Screen::EMPTY_CELL; buffer_size].into_boxed_slice();
        let cur_screen = vec![Screen::EMPTY_CELL; buffer_size].into_boxed_slice();

        let cursor_x = 0;
        let cursor_y = 0;

        let stored_x = 0;
        let stored_y = 0;

        let fg_color = Color::Reset;
        let bg_color = Color::Reset;

        Ok(Self {
            new_screen,
            cur_screen,
            width,
            height,
            cursor_x,
            cursor_y,
            stored_x,
            stored_y,
            force_redraw: false,
            redraws: 0,
            fg_color,
            bg_color,
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

    pub fn save_position(&mut self) {
        self.stored_x = self.cursor_x;
        self.stored_y = self.cursor_y;
    }

    pub fn restore_position(&mut self) {
        self.cursor_x = self.stored_x;
        self.cursor_y = self.stored_y;
    }

    pub fn clear(&mut self, ty: ClearType) {
        match ty {
            ClearType::All => {
                self.new_screen.fill(Screen::EMPTY_CELL);
            },
            ClearType::CurrentLine => {
                let start = self.cursor_y as usize * self.width as usize;
                let end = start + self.width as usize;
                self.new_screen[start..end].fill(Screen::EMPTY_CELL);
            },
            ClearType::UntilNewline => {
                let line_start = self.cursor_y as usize * self.width as usize;
                let start = line_start + self.cursor_x as usize;
                let end = line_start + self.width as usize;
                self.new_screen[start..end].fill(Screen::EMPTY_CELL);
            },
            ClearType::Current => {
                self.set_buffer_at(self.cursor_x as usize, self.cursor_y as usize, '\0');
            },
        }
    }

    pub fn print<S: AsRef<str>>(&mut self, message: S) {
        let mut x = self.cursor_x as usize;
        let mut y = self.cursor_y as usize;
        let width = self.width as usize;
        let height = self.height as usize;

        message
            .as_ref()
            .chars()
            .for_each(|c| {
                match c {
                    '\n' => {
                        x = 0;
                        y += 1;
                        if y >= height {
                            y = 0;
                        }
                    },
                    c => {
                        let index = y * width + x;
                        self.new_screen[index].fg_color = self.fg_color;
                        self.new_screen[index].bg_color = self.bg_color;
                        self.new_screen[index].c = c;

                        x += 1;
                        if x >= width {
                            x = 0;
                            y += 1;
                            if y >= height {
                                y = 0;
                            }
                        }
                    },
                }
            });

        self.cursor_x = x.try_into().unwrap();
        self.cursor_y = y.try_into().unwrap();
    }

    pub fn print_char(&mut self, c: char) {
        match c {
            '\n' => {
                self.cursor_x = 0;
                self.cursor_y += 1;
                if self.cursor_y >= self.height {
                    self.cursor_y = 0;
                }
            },
            c => {
                // Override cell
                let index = self.cursor_y as usize * self.width as usize + self.cursor_x as usize;
                self.new_screen[index].fg_color = self.fg_color;
                self.new_screen[index].bg_color = self.bg_color;
                self.new_screen[index].c = c;

                self.cursor_x += 1;
                if self.cursor_x >= self.width {
                    self.cursor_x = 0;
                    self.cursor_y += 1;
                    if self.cursor_y >= self.height {
                        self.cursor_y = 0;
                    }
                }
            },
        }
    }

    pub fn print_at<S: AsRef<str>>(&mut self, x: u16, y: u16, message: S) {
        self.move_to(x, y);
        self.print(message);
    }

    pub fn move_to(&mut self, x: u16, y: u16) {
        self.cursor_x = x.clamp(0, self.width - 1);
        self.cursor_y = y.clamp(0, self.height - 1);
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
        // swap width
        let old_width = self.width;
        let old_height = self.height;
        self.width = width;
        self.height = height;

        // create new buffer
        let mut new_buffer = vec![Screen::EMPTY_CELL; width as usize * height as usize].into_boxed_slice();
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
        self.cursor_x = self.cursor_x.min(width - 1);
        self.cursor_y = self.cursor_y.min(height - 1);
        self.stored_x = self.stored_x.min(width - 1);
        self.stored_y = self.stored_y.min(height - 1);

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
                    self.redraws += 1;
                    let x = start % width;
                    let y = start / width;
                    stdout.queue(MoveTo(x as u16, y as u16))?;
                    stdout.queue(Print(self.new_screen[start..i].iter().map(|cell| cell.c).collect::<String>()))?;
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
            self.redraws += 1;
            let x = start % width;
            let y = start / width;
            stdout.queue(MoveTo(x as u16, y as u16))?;
            stdout.queue(Print(self.new_screen[start..i].iter().map(|cell| cell.c).collect::<String>()))?;
        }
        stdout.queue(MoveTo(self.cursor_x, self.cursor_y))?;
        stdout.flush()?;

        Ok(())
    }

    pub fn show(&mut self) -> io::Result<()> {
        let mut stdout = stdout();

        self.redraws = 0;

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
                        self.redraws += 1;
                        let x = start % width;
                        let y = start / width;
                        stdout.queue(MoveTo(x as u16, y as u16))?;
                        stdout.queue(Print(self.new_screen[start..i].iter().map(|cell| cell.c).collect::<String>()))?;
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
                            self.redraws += 1;
                            let x = start % width;
                            let y = start / width;
                            stdout.queue(MoveTo(x as u16, y as u16))?;
                            stdout.queue(Print(self.new_screen[start..i].iter().map(|cell| cell.c).collect::<String>()))?;
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
                self.redraws += 1;
                let x = start % width;
                let y = start / width;
                stdout.queue(MoveTo(x as u16, y as u16))?;
                stdout.queue(Print(self.new_screen[start..i].iter().map(|cell| cell.c).collect::<String>()))?;
            }
            stdout.queue(MoveTo(self.cursor_x, self.cursor_y))?;
            stdout.flush()?;
        }

        self.force_redraw = false;

        self.cur_screen.copy_from_slice(&self.new_screen);

        Ok(())
    }

    pub fn get_redraws(&mut self) -> usize {
        self.redraws
    }

    fn set_buffer_at(&mut self, x: usize, y: usize, value: char) {
        let index = y * self.width as usize + x;
        self.new_screen[index].c = value;
        /* println!("Index: {}", index);
        println!("Value: {} :: {}", value, self.buffer[index].value); */
    }
}
