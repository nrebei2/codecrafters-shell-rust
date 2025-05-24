use std::{
    cmp::max,
    fmt::Display,
    io::{self, Stdout, Write},
};

use termion::{
    clear, cursor,
    raw::{IntoRawMode, RawTerminal},
};

use crate::{
    command_trie::{CommandsTrie, CompletionResponse},
    history::History,
};

enum Selected<'a> {
    History { index: usize, input: &'a str },
    Input,
}

struct InputDisplay<'a> {
    input: String,
    selected: Selected<'a>,
}

impl<'a> InputDisplay<'a> {
    fn move_up(&mut self, history: &'a History) {
        self.selected = match self.selected {
            Selected::Input => {
                if let Some(input) = history.back() {
                    Selected::History {
                        index: history.len() - 1,
                        input,
                    }
                } else {
                    Selected::Input
                }
            }
            Selected::History { index, .. } => {
                let index = index.saturating_sub(1);
                Selected::History {
                    index,
                    input: &history[index],
                }
            }
        };
    }

    fn move_down(&mut self, history: &'a History) {
        self.selected = match self.selected {
            Selected::Input => Selected::Input,
            Selected::History { index, .. } => {
                if index == history.len() - 1 {
                    Selected::Input
                } else {
                    let index = index + 1;
                    Selected::History {
                        index,
                        input: &history[index],
                    }
                }
            }
        };
    }

    fn modify_input(&mut self) -> &mut String {
        match self.selected {
            Selected::Input => &mut self.input,
            Selected::History { input, .. } => {
                self.input = input.to_owned();
                &mut self.input
            }
        }
    }

    fn cur_input(&self) -> &str {
        match self.selected {
            Selected::Input => &self.input,
            Selected::History { input, .. } => input,
        }
    }
}

pub struct InputState<'a> {
    input_display: InputDisplay<'a>,
    cursor_pos: usize,
    raw: RawTerminal<Stdout>,
    rang_bell: bool,
}

impl<'a> InputState<'a> {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            input_display: InputDisplay {
                input: String::new(),
                selected: Selected::Input,
            },
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
        match command_trie.autocomplete(self.input_display.cur_input()) {
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
                let input = self.input_display.modify_input();
                input.push_str(&rest);
                self.cursor_pos = input.len();
                self.print(&rest)
            }
            CompletionResponse::None => self.print('\x07'),
        }
    }

    pub fn handle_char(&mut self, c: char) -> io::Result<()> {
        self.rang_bell = false;

        let input = self.input_display.modify_input();
        input.insert(self.cursor_pos, c);
        self.cursor_pos += 1;

        if self.cursor_pos == input.len() {
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
        let input = self.input_display.modify_input();
        input.remove(self.cursor_pos);

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
        if self.cursor_pos == self.input_display.cur_input().len() {
            return Ok(());
        }

        self.cursor_pos += 1;

        self.print(cursor::Right(1))
    }

    pub fn handle_up(&mut self, history: &'a History) -> io::Result<()> {
        self.rang_bell = false;
        self.input_display.move_up(history);
        self.cursor_pos = self.input_display.cur_input().len();
        self.redraw()
    }

    pub fn handle_down(&mut self, history: &'a History) -> io::Result<()> {
        self.rang_bell = false;
        self.input_display.move_down(history);
        self.cursor_pos = self.input_display.cur_input().len();
        self.redraw()
    }

    pub fn submit(self) -> String {
        self.input_display.cur_input().to_owned()
    }

    fn put_cursor_end(&mut self) -> io::Result<()> {
        let input = self.input_display.cur_input();
        let move_by = (input.len() - self.cursor_pos) as u16;

        if move_by != 0 {
            write!(self.raw, "{}", cursor::Right(move_by))?;
            self.raw.flush()?;
        }

        self.cursor_pos = input.len();
        Ok(())
    }

    fn print<T: Display>(&mut self, s: T) -> io::Result<()> {
        write!(self.raw, "{s}")?;
        self.raw.flush()
    }

    fn redraw(&mut self) -> io::Result<()> {
        write!(self.raw, "\r{}", clear::CurrentLine)?;
        self.begin()?;

        let input = self.input_display.cur_input();
        write!(self.raw, "{}", input)?;

        let move_by = (input.len() - self.cursor_pos) as u16;

        if move_by != 0 {
            write!(self.raw, "{}", cursor::Left(move_by))?;
        }

        self.raw.flush()
    }
}
