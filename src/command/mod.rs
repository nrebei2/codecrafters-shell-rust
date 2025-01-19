use std::{
    env,
    fs::File,
    io::{stderr, stdin, stdout, Read, Write},
    path::PathBuf,
    process::{exit, Stdio},
    str::FromStr,
};

use std::process::Command as ProcessCommand;

mod parser;
use parser::{Command as PCommand, CommandParser, Fd, RedirectTo, RedirectType};

enum InternalCommandName {
    Echo,
    Type,
    Cd,
    Empty,
    Exit,
    Pwd,
}

impl FromStr for InternalCommandName {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "" => Self::Empty,
            "echo" => Self::Echo,
            "type" => Self::Type,
            "cd" => Self::Cd,
            "exit" => Self::Exit,
            "pwd" => Self::Pwd,
            _ => return Err("nuh uh"),
        })
    }
}

struct InternalCommand {
    name: InternalCommandName,
    args: Vec<String>,
    input: Box<dyn Read>,
    output: Box<dyn Write>,
    error: Box<dyn Write>,
}

fn new_file(r_type: RedirectType, file_name: String) -> File {
    File::options()
        .append(matches!(r_type, RedirectType::Append))
        .write(true)
        .create(true)
        .open(file_name)
        .expect("Could not open file for redirection")
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

impl InternalCommand {
    fn from_parsed_command(comm: PCommand) -> Result<Self, PCommand> {
        let name = match comm.name.parse() {
            Ok(n) => n,
            Err(_) => return Err(comm),
        };

        let (output, error): (Box<dyn Write>, Box<dyn Write>) = match comm.redirect {
            None => (Box::new(stdout()), Box::new(stderr())),
            Some(r) => {
                if let RedirectTo::File(file_name) = r.to {
                    let file = new_file(r.r_type, file_name);
                    match r.from {
                        Fd::Stdout => (Box::new(file), Box::new(stderr())),
                        Fd::Stderr => (Box::new(stdout()), Box::new(file)),
                        _ => unimplemented!("Internal commands do not work with arbitrary fds"),
                    }
                } else {
                    unimplemented!("Only redirections to files are supported atm")
                }
            }
        };

        Ok(InternalCommand {
            name,
            args: comm.args,
            input: Box::new(stdin()), // piping not supported, i would have to start worrying about spsc channels to connect internal commands
            output,
            error,
        })
    }

    fn run(mut self) {
        match self.name {
            InternalCommandName::Exit => exit(0),
            InternalCommandName::Echo => {
                let _ = writeln!(self.output, "{}", self.args.join(" "));
            }
            InternalCommandName::Type => {
                let _ = match self.args.get(0).map(String::as_str) {
                    Some(comm @ ("echo" | "cd" | "type" | "exit" | "pwd")) => {
                        writeln!(self.output, "{comm} is a shell builtin")
                    }
                    Some(comm) => match find_in_path(&comm) {
                        Some(full_path) => {
                            writeln!(self.output, "{comm} is {}", full_path.display())
                        }
                        None => writeln!(self.error, "{comm}: not found"),
                    },
                    None => writeln!(self.error, "Expected an arguement"),
                };
            }
            InternalCommandName::Pwd => {
                if !self.args.is_empty() {
                    let _ = writeln!(self.error, "expected 0 arguments; got {}", self.args.len());
                    return;
                }
                let _ = match env::current_dir() {
                    Ok(path) => writeln!(self.output, "{}", path.display()),
                    _ => writeln!(self.error, "Current directory cannot be found!"),
                };
            }
            InternalCommandName::Cd => {
                if self.args.len() > 1 {
                    let _ = writeln!(self.error, "Too many args for cd command");
                    return;
                }

                if self.args.is_empty() {
                    self.args.push("~".into());
                }

                let mut path_str = self.args.pop().unwrap();
                if path_str.starts_with('~') {
                    #[allow(deprecated)]
                    let home = std::env::home_dir().unwrap();
                    let home_expanded = path_str.replacen('~', &home.display().to_string(), 1);
                    path_str = home_expanded;
                }

                if env::set_current_dir(&path_str).is_err() {
                    let _ = writeln!(self.error, "cd: {}: No such file or directory", path_str);
                }
            }
            InternalCommandName::Empty => {}
        }
    }
}

struct ExternalCommand {
    process: ProcessCommand,
}

impl ExternalCommand {
    fn from_parsed_command(comm: PCommand) -> Self {
        let mut process = ProcessCommand::new(comm.name);
        process.args(comm.args);

        if let Some(r) = comm.redirect {
            if let RedirectTo::File(file_name) = r.to {
                let file = new_file(r.r_type, file_name);
                match r.from {
                    Fd::Stdout => {
                        process.stdout(Stdio::from(file));
                    }
                    Fd::Stderr => {
                        process.stderr(Stdio::from(file));
                    }
                    _ => unimplemented!("Internal commands do not work with arbitrary fds"),
                }
            } else {
                // pre_exec could be used here
                unimplemented!("Only redirections to files are supported atm")
            }
        }

        ExternalCommand { process }
    }

    fn run(mut self) {
        match self.process.spawn() {
            Ok(mut child) => {
                child.wait().expect("command wasn't running");
            }
            Err(_) => {
                let _ = writeln!(
                    stderr(),
                    "{}: command not found",
                    self.process.get_program().to_str().unwrap()
                );
            }
        };
    }
}

pub fn run_from_input(input: &str) {
    let p_comm = CommandParser::new(input).parse();

    match InternalCommand::from_parsed_command(p_comm) {
        Ok(internal_comm) => internal_comm.run(),
        Err(p_comm) => ExternalCommand::from_parsed_command(p_comm).run(),
    }
}
