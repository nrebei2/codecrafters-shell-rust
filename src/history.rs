use std::{cmp::max, collections::VecDeque, fmt::Display, ops::Deref};

#[derive(Default)]
pub struct History {
    inputs: VecDeque<String>,
}

impl History {
    pub fn push(&mut self, input: String) {
        self.inputs.push_back(input);
    }
}

impl Deref for History {
    type Target = VecDeque<String>;

    fn deref(&self) -> &Self::Target {
        &self.inputs
    }
}

impl Display for History {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, input) in self.inputs.iter().enumerate() {
            writeln!(f, "\t{}  {}", idx + 1, input)?;
        }
        Ok(())
    }
}
