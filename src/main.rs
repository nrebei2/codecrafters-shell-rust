use std::{fmt::Display, io::Read, process::exit};
#[allow(unused_imports)]
use std::io::{self, Write};
use termion::raw::IntoRawMode;

use sequence_trie::SequenceTrie;
mod command;

fn print<T: Display>(s: T) {
    print!("{s}");
    io::stdout().flush().unwrap();
}

fn main() {
    let trie = build_trie();

    let mut stdin = io::stdin();
    let mut input = String::new();
    let mut byte_buf = [0];

    loop {
        let raw = io::stdout().into_raw_mode().unwrap();

        print("$ ");
        loop {
            stdin.read_exact(&mut byte_buf).unwrap();

            match byte_buf[0] as char {
                '\n' | '\r' => {
                    print("\r\n");
                    break;
                },
                '\t' => {
                    match trie.autocomplete(&input) {
                        CompletionResponse::Single(mut rest) => {
                            rest.push(' ');
                            print(&rest);
                            input.push_str(&rest);
                        }
                        _ => todo!()
                    }
                },
                '\x7F' => { // sent on 'delete'
                    print("\x08 \x08"); // move back - overwrite - move back
                    input.pop();
                }
                c => {
                    print(c);
                    input.push(c);
                }
            }
        }

        drop(raw);
        command::run_from_input(&input);
        input.clear();
    }
}

fn build_trie() -> CommandsTrie {
    let mut trie = CommandsTrie::new();
    trie.insert("echo");
    trie.insert("exit");
    trie
}

struct CommandsTrie {
    trie: SequenceTrie<u8, ()>,
}

enum CompletionResponse {
    None,
    Single(String),
    Multiple(Vec<String>),
}

impl CommandsTrie {
    fn new() -> Self {
        Self {
            trie: SequenceTrie::new(),
        }
    }

    fn insert(&mut self, command_name: &str) {
        self.trie.insert(command_name.as_bytes(), ());
    }

    fn autocomplete(&self, current: &str) -> CompletionResponse {
        let mut string_builder = vec![];
        if let Some(mut cur_node) = self.trie.get_node(current.as_bytes()) {
            loop {
                if cur_node.value().is_some() {
                    return CompletionResponse::Single(String::from_utf8(string_builder).unwrap());
                }

                let children = cur_node.children_with_keys();
                match children.as_slice() {
                    [] => break,
                    [(&byte, single_child)] => {
                        string_builder.push(byte);
                        cur_node = single_child;
                    }
                    _ => {
                        let completions = cur_node.keys().map(|postfix| {
                            let mut new_string = string_builder.clone();
                            new_string.extend(postfix);
                            String::from_utf8(new_string).unwrap()
                        });
                        return CompletionResponse::Multiple(completions.collect());
                    }
                }
            }
        };
        CompletionResponse::None
    }
}
