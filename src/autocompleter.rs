use std::{env, fs, os::unix::fs::PermissionsExt};

use sequence_trie::SequenceTrie;

pub enum CompletionResponse {
    None,
    Single(String, Option<&'static str>),  // does not include prefix, includes postpend
    Multiple(Vec<String>), // holds all matches including prefix
}

pub struct Autocompleter {
    trie: SequenceTrie<u8, Option<&'static str>>,
}

impl Autocompleter {
    pub fn new() -> Self {
        Self {
            trie: SequenceTrie::new(),
        }
    }

    pub fn insert_val(&mut self, name: &str, val: &'static str) {
        self.trie.insert(name.as_bytes(), Some(val));
    }

    pub fn insert(&mut self, name: &str) {
        self.trie.insert(name.as_bytes(), None);
    }

    pub fn autocomplete(&self, current: &str) -> CompletionResponse {
        let mut string_builder = vec![];
        if let Some(mut cur_node) = self.trie.get_node(current.as_bytes()) {
            loop {
                if let Some(val) = cur_node.value() {
                    return CompletionResponse::Single(
                        String::from_utf8(string_builder).unwrap(),
                        if cur_node.is_leaf() { *val } else { Some("") },
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
                        let completions = cur_node.iter().map(|(postfix, val)| {
                            let mut new_string = current.as_bytes().to_vec();
                            new_string.extend(&string_builder);
                            new_string.extend(postfix);
                            String::from_utf8(new_string).unwrap() + val.unwrap_or_default()
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

pub fn build_command_completer() -> Autocompleter {
    let mut completer = Autocompleter::new();
    completer.insert("echo");
    completer.insert("exit");
    for path in env::split_paths(&env::var_os("PATH").unwrap()) {
        for entry in path
            .read_dir()
            .into_iter()
            .flat_map(|r| r.into_iter())
            .flatten()
        {
            if entry.path().is_file()
                && fs::metadata(entry.path()).unwrap().permissions().mode() & 0o111 != 0
            {
                completer.insert(&entry.file_name().into_string().unwrap());
            }
        }
    }
    completer
}
