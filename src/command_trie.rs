use std::{env, fs, os::unix::fs::PermissionsExt};

use sequence_trie::SequenceTrie;

pub struct CommandsTrie {
    trie: SequenceTrie<u8, ()>,
}

pub enum CompletionResponse {
    None,
    Single(String, bool),  // does not include prefix
    Multiple(Vec<String>), // holds all matches including prefix
}

impl CommandsTrie {
    pub fn new() -> Self {
        Self {
            trie: SequenceTrie::new(),
        }
    }

    pub fn insert(&mut self, command_name: &str) {
        self.trie.insert(command_name.as_bytes(), ());
    }

    pub fn autocomplete(&self, current: &str) -> CompletionResponse {
        let mut string_builder = vec![];
        if let Some(mut cur_node) = self.trie.get_node(current.as_bytes()) {
            loop {
                if cur_node.value().is_some() {
                    return CompletionResponse::Single(
                        String::from_utf8(string_builder).unwrap(),
                        cur_node.is_leaf(),
                    );
                }

                let children = cur_node.children_with_keys();
                match children.as_slice() {
                    [] => break,
                    [(&byte, single_child)] => {
                        string_builder.push(byte);
                        cur_node = single_child;
                    }
                    _ => {
                        // branches off, return all sub-keys
                        let completions = cur_node.keys().map(|postfix| {
                            let mut new_string = current.as_bytes().to_vec();
                            new_string.extend(&string_builder);
                            new_string.extend(postfix);
                            String::from_utf8(new_string).unwrap()
                        });
                        let mut commands: Vec<_> = completions.collect();
                        commands.sort();
                        return CompletionResponse::Multiple(commands);

                    }
                }
            }
        };
        CompletionResponse::None
    }
}

pub fn build_trie() -> CommandsTrie {
    let mut trie = CommandsTrie::new();
    trie.insert("echo");
    trie.insert("exit");
    for path in env::split_paths(&env::var_os("PATH").unwrap()) {
        for entry in path.read_dir().into_iter().flat_map(|r| r.into_iter()) {
            if let Ok(entry) = entry {
                if entry.path().is_file()
                    && fs::metadata(entry.path()).unwrap().permissions().mode() & 0o111 != 0
                {
                    trie.insert(&entry.file_name().into_string().unwrap());
                }
            }
        }
    }
    trie
}
