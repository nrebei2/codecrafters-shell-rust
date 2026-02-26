use std::{
    env,
    fs::File,
    io::{stderr, stdin, stdout, Read, Write},
    path::PathBuf,
    process::Stdio,
    str::FromStr,
    sync::Mutex,
};

use std::process::Command as ProcessCommand;

mod parser;
use is_executable::is_executable;
use parser::{Command as PCommand, CommandParser, Fd, RedirectTo, RedirectType};

use crate::history::History;

#[derive(PartialEq)]
enum InternalCommandName {
    Echo,
    Type,
    Cd,
    Empty,
    Exit,
    Pwd,
    History,
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
            "history" => Self::History,
            _ => return Err("nuh uh"),
        })
    }
}

struct InternalCommand {
    name: InternalCommandName,
    args: Vec<String>,
    input: Box<dyn Read + Send>,
    output: Box<dyn Write + Send>,
    error: Box<dyn Write + Send>,
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
        if joined.is_file() && is_executable(&joined) {
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

        let (output, error): (Box<dyn Write + Send>, Box<dyn Write + Send>) = match comm.redirect {
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
            input: Box::new(stdin()),
            output,
            error,
        })
    }

    fn run(mut self, history: &Mutex<History>) {
        match self.name {
            InternalCommandName::Echo => {
                let _ = writeln!(self.output, "{}", self.args.join(" "));
            }
            InternalCommandName::Type => {
                let _ = match self.args.first().map(String::as_str) {
                    Some(comm @ ("echo" | "cd" | "type" | "exit" | "pwd" | "history")) => {
                        writeln!(self.output, "{comm} is a shell builtin")
                    }
                    Some(comm) => match find_in_path(comm) {
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
            InternalCommandName::History => {
                let mut history = history.lock().unwrap();
                let _ = match self.args.first().map(String::as_str) {
                    None => history.write(&mut self.output, None),
                    Some("-r") => {
                        let Some(path) = self.args.get(1) else {
                            let _ =
                                writeln!(self.error, "history -r: Expected <path_to_history_file>");
                            return;
                        };

                        let Some(file_history) = History::from_file(path.into()) else {
                            let _ = writeln!(self.error, "history -r {path}: Could not read file");
                            return;
                        };

                        *history += file_history;

                        Ok(())
                    }
                    Some(arg @ ("-w" | "-a")) => {
                        let Some(path) = self.args.get(1) else {
                            let _ = writeln!(
                                self.error,
                                "history {arg}: Expected <path_to_history_file>"
                            );
                            return;
                        };

                        if let Err(e) = history.write_to_file(path.into(), arg == "-a") {
                            let _ = writeln!(
                                self.error,
                                "history {arg} {path}: Could not create/write file - {}",
                                e
                            );
                            return;
                        };

                        Ok(())
                    }
                    Some(arg) => {
                        let Ok(limit) = arg.parse::<usize>() else {
                            let _ = writeln!(self.error, "history {}: Invalid option", arg);
                            return;
                        };

                        history.write(&mut self.output, Some(limit))
                    }
                };
            }
            InternalCommandName::Exit => {}
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
            let RedirectTo::File(file_name) = r.to else {
                // pre_exec could be used here
                unimplemented!("Only redirections to files are supported atm")
            };
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

enum Command {
    Internal(InternalCommand),
    External(ExternalCommand),
}

#[derive(PartialEq)]
pub enum RunResult {
    Exit,
    Continue,
}

pub fn run_from_history(history: &Mutex<History>) -> RunResult {
    // input retrieved from end of history
    let binding = history.lock().unwrap();
    let input = binding.last().unwrap();
    let parsed_commands = CommandParser::new(input).parse();
    drop(binding);

    if parsed_commands.is_empty() {
        return RunResult::Continue;
    }

    let mut res = RunResult::Continue;

    let mut compiled_commands: Vec<_> = parsed_commands
        .into_iter()
        .map(|p_c| match InternalCommand::from_parsed_command(p_c) {
            Ok(internal_comm) => {
                if internal_comm.name == InternalCommandName::Exit {
                    res = RunResult::Exit;
                };
                Command::Internal(internal_comm)
            }
            Err(p_comm) => Command::External(ExternalCommand::from_parsed_command(p_comm)),
        })
        .collect();

    // pipe each pair of adjacent commands together
    for i in 0..compiled_commands.len() - 1 {
        match &mut compiled_commands[i..i + 2] {
            [Command::External(e_1), Command::External(e_2)] => {
                let (reader, writer) = os_pipe::pipe().unwrap();

                e_1.process.stdout(writer);
                e_2.process.stdin(reader);
            }
            [Command::External(ec), Command::Internal(ic)] => {
                let (reader, writer) = os_pipe::pipe().unwrap();

                ec.process.stdout(writer);
                ic.input = Box::new(reader);
            }
            [Command::Internal(ic), Command::External(ec)] => {
                let (reader, writer) = os_pipe::pipe().unwrap();

                ic.output = Box::new(writer);
                ec.process.stdin(reader);
            }
            [Command::Internal(i_1), Command::Internal(i_2)] => {
                let (reader, writer) = pipe::pipe();

                i_1.output = Box::new(writer);
                i_2.input = Box::new(reader);
            }
            _ => unreachable!(),
        }
    }

    // run commands on separate threads
    std::thread::scope(|s| {
        for comm in compiled_commands {
            s.spawn(|| match comm {
                Command::External(e) => e.run(),
                Command::Internal(i) => i.run(history),
            });
        }
    });

    res
}
