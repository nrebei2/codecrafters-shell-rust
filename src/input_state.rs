use std::{
    fmt::Display,
    io::{self, Stdout, Write},
};

use termion::{
    clear, cursor,
    raw::{IntoRawMode, RawTerminal},
};

use crate::command_trie::{CommandsTrie, CompletionResponse};

pub struct InputState {
    input: String,
    cursor_pos: usize,
    raw: RawTerminal<Stdout>,
    rang_bell: bool,
}

impl InputState {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            input: String::new(),
            cursor_pos: 0,
            raw: io::stdout().into_raw_mode()?,
            rang_bell: false,
        })
    }

    pub fn begin(&mut self) -> io::Result<()> {
        self.print("$ ")
    }

    pub fn handle_newline(&mut self) -> io::Result<()> {
        self.rang_bell = false;
        self.print("\r\n")
    }

    pub fn handle_tab(&mut self, command_trie: &CommandsTrie) -> io::Result<()> {
        match command_trie.autocomplete(&self.input) {
            CompletionResponse::Multiple(matches) => {
                if self.rang_bell {
                    self.rang_bell = false;
                    self.print("\r\n")?;
                    self.print(matches.join("  "))?;
                    self.print("\n")?;
                    self.redraw()
                } else {
                    self.rang_bell = true;
                    self.print('\x07')
                }
            }
            CompletionResponse::Single(mut rest, is_leaf) => {
                if is_leaf {
                    rest.push(' ');
                }
                self.put_cursor_end()?;
                self.input.push_str(&rest);
                self.cursor_pos = self.input.len();
                self.print(&rest)
            }
            CompletionResponse::None => self.print('\x07'),
        }
    }

    pub fn handle_char(&mut self, c: char) -> io::Result<()> {
        self.rang_bell = false;

        self.input.insert(self.cursor_pos, c);
        self.cursor_pos += 1;

        if self.cursor_pos == self.input.len() {
            self.print(c)
        } else {
            self.redraw()
        }
    }

    pub fn handle_backspace(&mut self) -> io::Result<()> {
        self.rang_bell = false;
        if self.cursor_pos == 0 {
            return Ok(());
        }

        self.cursor_pos -= 1;
        self.input.remove(self.cursor_pos);

        self.redraw()
    }

    pub fn handle_left(&mut self) -> io::Result<()> {
        self.rang_bell = false;
        if self.cursor_pos == 0 {
            return Ok(());
        }

        self.cursor_pos -= 1;

        self.print(cursor::Left(1))
    }

    pub fn handle_right(&mut self) -> io::Result<()> {
        self.rang_bell = false;
        if self.cursor_pos == self.input.len() {
            return Ok(());
        }

        self.cursor_pos += 1;

        self.print(cursor::Right(1))
    }

    pub fn submit(self) -> String {
        self.input
    }
    
    fn put_cursor_end(&mut self) -> io::Result<()> {
        let move_by = (self.input.len() - self.cursor_pos) as u16;

        if move_by != 0 {
            write!(self.raw, "{}", cursor::Right(move_by))?;
            self.raw.flush()?;
        } 

        self.cursor_pos = self.input.len();
        Ok(())
    }

    fn print<T: Display>(&mut self, s: T) -> io::Result<()> {
        write!(self.raw, "{s}")?;
        self.raw.flush()
    }

    fn redraw(&mut self) -> io::Result<()> {
        write!(self.raw, "\r{}", clear::CurrentLine)?;
        self.begin()?;
        write!(self.raw, "{}", &self.input)?;

        let move_by = (self.input.len() - self.cursor_pos) as u16;

        if move_by != 0 {
            write!(self.raw, "{}", cursor::Left(move_by))?;
        }

        self.raw.flush()
    }
}
