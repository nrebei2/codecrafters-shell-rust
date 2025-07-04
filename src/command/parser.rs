use std::{
    iter::Peekable,
    num::ParseIntError,
    str::{Chars, FromStr},
};

#[derive(Debug)]
pub enum Fd {
    Stdin,
    Stdout,
    Stderr,
    Other(i32),
}

impl FromStr for Fd {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s.is_empty() {
            Self::Stdout
        } else {
            match i32::from_str(s)? {
                1 => Self::Stdout,
                2 => Self::Stderr,
                oth => Self::Other(oth),
            }
        })
    }
}

#[derive(Debug)]
pub enum RedirectTo {
    File(String),
    Fd(Fd),
}

#[derive(Debug)]
pub enum RedirectType {
    Normal,
    Append,
}

#[derive(Debug)]
pub struct Redirect {
    pub r_type: RedirectType,
    pub from: Fd,
    pub to: RedirectTo,
}

#[derive(Default, Debug)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    pub redirect: Option<Redirect>,
}

/// Follows single/double quote rules
pub struct CommandParser<'a> {
    chars: Peekable<Chars<'a>>,
    buf: String,
}

impl<'a> CommandParser<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            chars: s.chars().peekable(),
            buf: String::new(),
        }
    }

    fn parse_single_quotes(&mut self) {
        for c in self.chars.by_ref() {
            if c == '\'' {
                break;
            }
            self.buf.push(c);
        }
    }

    fn parse_double_quotes(&mut self) {
        while let Some(c) = self.chars.next() {
            match c {
                '\"' => break,
                '\\' => match self.chars.next().unwrap() {
                    n @ ('\\' | '$' | '"') => {
                        self.buf.push(n);
                    }
                    oth => {
                        self.buf.push('\\');
                        self.buf.push(oth)
                    }
                },
                _ => self.buf.push(c),
            }
        }
    }

    fn parse_string(&mut self) {
        if self.chars.peek().is_none() && self.buf.is_empty() {
            panic!("Expected string, found end of input")
        }

        while let Some(c) = self.chars.next() {
            if c.is_ascii_whitespace() {
                break;
            }

            match c {
                '>' => {
                    panic!("Expected a string, but found a redirection")
                }
                '|' => {
                    panic!("Expected a string, but found a pipe")
                }
                '\\' => self.buf.push(self.chars.next().unwrap()),
                '\'' => self.parse_single_quotes(),
                '"' => self.parse_double_quotes(),
                _ => self.buf.push(c),
            }
        }
    }

    fn try_parse_redirect(&mut self) -> Option<Redirect> {
        while let Some(c) = self.chars.next_if(|c| c.is_ascii_digit()) {
            self.buf.push(c);
        }

        match self.chars.peek() {
            Some('>') => {
                self.chars.next(); // >
                let from = Fd::from_str(&self.buf).expect("Expected a valid file descriptor");
                self.buf.clear();

                let r_type = match self.chars.peek() {
                    Some('>') => {
                        self.chars.next(); // >
                        RedirectType::Append
                    }
                    _ => RedirectType::Normal,
                };

                let to = match self.chars.peek() {
                    Some('&') => {
                        self.chars.next(); // &
                        self.parse_string();
                        RedirectTo::Fd(
                            Fd::from_str(&self.buf).expect("Expected a valid file descriptor"),
                        )
                    }
                    _ => {
                        self.advance();
                        self.parse_string();
                        RedirectTo::File(self.buf.clone())
                    }
                };

                self.buf.clear();
                Some(Redirect { r_type, from, to })
            }
            _ => {
                // fallback
                None
            }
        }
    }

    pub fn parse_command(&mut self) -> Option<Command> {
        if self.advance() {
            return None; // empty string
        }

        let mut comm = Command {
            name: {
                self.parse_string();
                self.buf.drain(..).collect()
            },
            ..Default::default()
        };

        loop {
            if self.advance() {
                break;
            }

            if self.chars.peek() == Some(&'|') {
                break;
            }

            // check if redirection
            match self.try_parse_redirect() {
                Some(r) => comm.redirect = Some(r),
                None => {
                    self.parse_string();
                    comm.args.push(self.buf.drain(..).collect())
                }
            }
        }

        Some(comm)
    }

    /// returns a pipeline of commands, i.e. [c_1, c_2, ..., c_n] models 'c_1 | c_2 | ... | c_n'
    pub fn parse(mut self) -> Vec<Command> {
        let command = match self.parse_command() {
            None => return vec![],
            Some(c) => c,
        };
        let mut pipeline = vec![command];

        loop {
            match self.chars.next() {
                Some('|') => {
                    pipeline.push(
                        self.parse_command()
                            .expect("Expected a command to pipeline, but found end of input"),
                    );
                }
                None => break,
                _ => unreachable!(),
            }
        }
        pipeline
    }

    // true if exhausted iterator
    fn advance(&mut self) -> bool {
        while self.chars.next_if(|c| c.is_ascii_whitespace()).is_some() {}

        self.chars.peek().is_none()
    }
}

#[test]
fn test() {
    let parser = CommandParser::new("echo hello testing 2>&3 still an arg");
    dbg!(parser.parse());
}
