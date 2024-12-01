#[allow(unused_imports)]
use std::io::{self, Write};
use std::str::FromStr;

enum Command {
    Echo,
    Cd,
    Empty,
    Exit
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut blocks = s.split_whitespace();

        match blocks.next() {
            None => Ok(Self::Empty),
            Some(comm) => match &comm.to_ascii_lowercase()[..] {
                "echo" => Ok(Self::Echo),
                "cd" => Ok(Self::Cd),
                "exit" => Ok(Self::Exit),
                _ => Err(format!("{comm}: command not found")),
            },
        }
    }
}

fn print(s: &str) {
    print!("{s}");
    io::stdout().flush().unwrap();
}

fn main() {
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        print("$ ");
        stdin.read_line(&mut input).unwrap();

        match Command::from_str(&input) {
            Ok(comm) => {
                match comm {
                    Command::Exit => break,
                    _ => todo!()
                }
            }
            Err(e) => {
                println!("{e}");
            }
        }

        input.clear();
    }
}
