#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    env,
    iter::Peekable,
    path::PathBuf,
    process::Stdio,
    str::{Chars, FromStr, SplitWhitespace},
};

enum Command {
    Echo(String),
    Type(String),
    Cd(PathBuf),
    Empty,
    Exit,
    External(String, Vec<String>),
    Pwd,
}

fn collect(blocks: SplitArgs<'_>) -> String {
    let mut s = blocks.fold(String::new(), |mut a, b| {
        a.reserve(b.len() + 1);
        a.push_str(&b);
        a.push_str(" ");
        a
    });
    s.pop();
    s
}

struct SplitArgs<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> SplitArgs<'a> {
    fn parse_single_quotes(&mut self, str: &mut String) {
        while let Some(c) = self.chars.next() {
            if c == '\'' {
                break;
            }
            str.push(c);
        }
    }

    fn parse_double_quotes(&mut self, str: &mut String) {
        while let Some(c) = self.chars.next() {
            match c {
                '\"' => break,
                '\\' => match self.chars.next().unwrap() {
                    '\\' | '$' | '"' => {
                        str.push(self.chars.next().unwrap());
                    }
                    oth => {
                        str.push('\\');
                        str.push(oth)
                    }
                },
                _ => str.push(c),
            }
        }
    }

    fn parse_normal(&mut self) -> String {
        let mut res = String::new();

        while let Some(c) = self.chars.next() {
            if c.is_ascii_whitespace() {
                break;
            }

            match c {
                '\\' => res.push(self.chars.next().unwrap()),
                '\'' => self.parse_single_quotes(&mut res),
                '"' => self.parse_double_quotes(&mut res),
                _ => res.push(c),
            }
        }

        res
    }
}

impl<'a> Iterator for SplitArgs<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        while self.chars.next_if(|c| c.is_ascii_whitespace()).is_some() {}

        if let Some(_) = self.chars.peek() {
            Some(self.parse_normal())
        } else {
            None
        }
    }
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: will have to make a custom split_args iterator (Item=String)
        // that accounts for ', ", \
        let mut args = SplitArgs {
            chars: s.chars().peekable(),
        };

        match args.next() {
            None => Ok(Self::Empty),
            Some(comm) => match &comm.to_ascii_lowercase()[..] {
                "echo" => Ok(Self::Echo(collect(args))),
                "cd" => {
                    let mut path_str = args.next().ok_or("type: expected path")?.to_string();

                    if path_str.starts_with('~') {
                        #[allow(deprecated)]
                        let home = std::env::home_dir().unwrap();
                        let home_expanded = path_str.replacen('~', &home.display().to_string(), 1);
                        path_str = home_expanded;
                    }

                    Ok(Self::Cd(path_str.into()))
                }
                "exit" => Ok(Self::Exit),
                "type" => Ok(Self::Type(
                    args.next()
                        .ok_or("type: expected command")?
                        .to_ascii_lowercase(),
                )),
                "pwd" => Ok(Self::Pwd),
                _ => match find_in_path(&comm) {
                    Some(_) => Ok(Self::External(comm, args.collect())),
                    None => Err(format!("{comm}: command not found")),
                },
            },
        }
    }
}

fn print(s: &str) {
    print!("{s}");
    io::stdout().flush().unwrap();
}

fn find_in_path(comm: &str) -> Option<PathBuf> {
    for path in env::split_paths(&env::var_os("PATH").unwrap()) {
        let joined = path.join(comm);
        if joined.is_file() {
            return Some(joined);
        }
    }
    None
}

fn main() {
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        print("$ ");
        stdin.read_line(&mut input).unwrap();

        match Command::from_str(&input) {
            Ok(comm) => match comm {
                Command::Exit => break,
                Command::Echo(echo) => println!("{echo}"),
                Command::Type(comm) => match &comm[..] {
                    "echo" | "cd" | "type" | "exit" | "pwd" => {
                        println!("{comm} is a shell builtin")
                    }
                    _ => match find_in_path(&comm) {
                        Some(full_path) => println!("{comm} is {}", full_path.display()),
                        None => println!("{comm}: not found"),
                    },
                },
                Command::External(comm, args) => {
                    // TODO: just spawn the command directly, i have the full_path and args
                    std::process::Command::new(comm)
                        .args(args)
                        .stdout(Stdio::inherit())
                        .output()
                        .unwrap();
                }
                Command::Pwd => {
                    match env::current_dir() {
                        Ok(path) => println!("{}", path.display()),
                        _ => println!("Current directory cannot be found!"),
                    };
                }
                Command::Cd(path) => {
                    if env::set_current_dir(&path).is_err() {
                        println!("cd: {}: No such file or directory", path.display());
                    }
                }
                Command::Empty => {}
            },
            Err(e) => println!("{e}"),
        };

        input.clear();
    }
}
