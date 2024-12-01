#[allow(unused_imports)]
use std::io::{self, Write};
use std::{env, path::PathBuf, str::FromStr};

enum Command {
    Echo(String),
    Type(String),
    Cd,
    Empty,
    Exit,
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut blocks = s.split_whitespace();

        match blocks.next() {
            None => Ok(Self::Empty),
            Some(comm) => match &comm.to_ascii_lowercase()[..] {
                "echo" => {
                    let mut s = blocks.fold(String::new(), |mut a, b| {
                        a.reserve(b.len() + 1);
                        a.push_str(b);
                        a.push_str(" ");
                        a
                    });
                    s.pop();

                    Ok(Self::Echo(s))
                }
                "cd" => Ok(Self::Cd),
                "exit" => Ok(Self::Exit),
                "type" => Ok(Self::Type(
                    blocks
                        .next()
                        .ok_or("type: expected command")?
                        .to_ascii_lowercase(),
                )),
                _ => Err(format!("{comm}: command not found")),
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

        let response = match Command::from_str(&input) {
            Ok(comm) => match comm {
                Command::Exit => break,
                Command::Echo(echo) => echo,
                Command::Type(comm) => match &comm[..] {
                    "echo" | "cd" | "type" | "exit" => format!("{comm} is a shell builtin"),
                    _ => match find_in_path(&comm) {
                        Some(full_path) => format!("{comm} is {}", full_path.display()),
                        None => format!("{comm}: not found"),
                    },
                },
                _ => todo!(),
            },
            Err(e) => e,
        };

        println!("{response}");
        input.clear();
    }
}
