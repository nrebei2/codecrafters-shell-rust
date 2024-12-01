#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    env,
    io::Stdout,
    path::PathBuf,
    process::Stdio,
    str::{FromStr, SplitWhitespace},
};

enum Command {
    Echo(String),
    Type(String),
    Cd,
    Empty,
    Exit,
    External,
}

fn collect(blocks: SplitWhitespace<'_>) -> String {
    let mut s = blocks.fold(String::new(), |mut a, b| {
        a.reserve(b.len() + 1);
        a.push_str(b);
        a.push_str(" ");
        a
    });
    s.pop();
    s
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut blocks = s.split_whitespace();

        match blocks.next() {
            None => Ok(Self::Empty),
            Some(comm) => match &comm.to_ascii_lowercase()[..] {
                "echo" => Ok(Self::Echo(collect(blocks))),
                "cd" => Ok(Self::Cd),
                "exit" => Ok(Self::Exit),
                "type" => Ok(Self::Type(
                    blocks
                        .next()
                        .ok_or("type: expected command")?
                        .to_ascii_lowercase(),
                )),
                _ => match find_in_path(comm) {
                    Some(_) => Ok(Self::External),
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
                    "echo" | "cd" | "type" | "exit" => println!("{comm} is a shell builtin"),
                    _ => match find_in_path(&comm) {
                        Some(full_path) => println!("{comm} is {}", full_path.display()),
                        None => println!("{comm}: not found"),
                    },
                },
                Command::External => {
                    std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&input)
                        .stdout(Stdio::inherit())
                        .output()
                        .unwrap();
                }
                _ => todo!(),
            },
            Err(e) => println!("{e}"),
        };

        input.clear();
    }
}
