#[allow(unused_imports)]
use std::io::{self, Write};

mod command;

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

        command::run_from_input(&input);

        input.clear();
    }
}
