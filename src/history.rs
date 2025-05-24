use std::{cmp::max, collections::VecDeque, fmt::Display, io::Write, ops::Deref};

#[derive(Default)]
pub struct History {
    inputs: VecDeque<String>,
}

impl History {
    pub fn push(&mut self, input: String) {
        self.inputs.push_back(input);
    }

    pub fn write<W: Write>(
        &self,
        writer: &mut W,
        limit: Option<usize>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (idx, input) in self
            .inputs
            .iter()
            .enumerate()
            .skip(self.inputs.len().saturating_sub(limit.unwrap_or(self.inputs.len())))
        {
            writeln!(writer, "\t{}  {}", idx + 1, input)?;
        }
        Ok(())
    }
}

impl Deref for History {
    type Target = VecDeque<String>;

    fn deref(&self) -> &Self::Target {
        &self.inputs
    }
}