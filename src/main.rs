use command_trie::build_trie;
use input_state::InputState;
use std::{io, process::exit};
use termion::{event::Key, input::TermRead};

mod command;
mod command_trie;
mod input_state;

fn main() -> io::Result<()> {
    let trie = build_trie();

    loop {
        let mut input = InputState::new()?;
        input.begin()?;
        
        for key in io::stdin().keys().filter_map(Result::ok) {
            match key {
                Key::Char('\n') => {
                    input.handle_newline()?;
                    break;
                }
                Key::Char('\t') => input.handle_tab(&trie),
                Key::Char(c) => input.handle_char(c),
                Key::Backspace => input.handle_backspace(),
                Key::Left => input.handle_left(),
                Key::Right => input.handle_right(),
                Key::Ctrl('d') => exit(0),
                _ => Ok(())
            }?;
        }

        command::run_from_input(&input.submit());
    }
}
