#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    env,
    path::PathBuf,
    process::Stdio,
    str::{FromStr, SplitWhitespace},
};

enum Command {
    Echo(String),
    Type(String),
    Cd(PathBuf),
    Empty,
    Exit,
    External,
    Pwd,
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
        // TODO: will have to make a custom split_args iterator
        // that accounts for ', ", \
        let mut blocks = s.split_whitespace();

        match blocks.next() {
            None => Ok(Self::Empty),
            Some(comm) => match &comm.to_ascii_lowercase()[..] {
                "echo" => Ok(Self::Echo(collect(blocks))),
                "cd" => {
                    let mut path_str = blocks.next().ok_or("type: expected path")?.to_string();

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
                    blocks
                        .next()
                        .ok_or("type: expected command")?
                        .to_ascii_lowercase(),
                )),
                "pwd" => Ok(Self::Pwd),
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
                    // TODO: just spawn the command directly, i have the full_path and args
                    std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&input)
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
