use command_trie::build_trie;
use history::History;
use input_state::InputState;
use std::{io, process::exit, sync::Mutex};
use termion::{event::Key, input::TermRead};

mod command;
mod command_trie;
mod history;
mod input_state;

fn main() -> io::Result<()> {
    let trie = build_trie();
    let mut history = Mutex::new(History::default());

    loop {
        let mut input = InputState::new()?;
        input.begin()?;

        let history_handle = history.get_mut().unwrap();

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
                Key::Up => input.handle_up(history_handle),
                Key::Down => input.handle_down(history_handle),
                Key::Ctrl('d') => exit(0),
                _ => Ok(()),
            }?;
        }

        history_handle.push(input.submit());
        command::run_from_history(&history);
    }
}
