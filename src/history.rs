use std::{
    fs,
    io::{BufRead, BufReader, Write},
    ops::{AddAssign, Deref},
    path::PathBuf,
};

#[derive(Default)]
pub struct History {
    inputs: Vec<String>,
}

impl History {
    pub fn push(&mut self, input: String) {
        self.inputs.push(input);
    }

    pub fn write<W: Write>(
        &self,
        writer: &mut W,
        limit: Option<usize>,
    ) -> Result<(), std::io::Error> {
        for (idx, input) in self.inputs.iter().enumerate().skip(
            self.inputs
                .len()
                .saturating_sub(limit.unwrap_or(self.inputs.len())),
        ) {
            writeln!(writer, "\t{}  {}", idx + 1, input)?;
        }
        Ok(())
    }

    pub fn from_file(file_path: PathBuf) -> Option<Self> {
        let inputs: Result<Vec<String>, _> = BufReader::new(fs::File::open(file_path).ok()?)
            .lines()
            .collect();

        Some(Self {
            inputs: inputs.ok()?,
        })
    }
}

impl Deref for History {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.inputs
    }
}

impl AddAssign for History {
    fn add_assign(&mut self, mut rhs: Self) {
        self.inputs.append(&mut rhs.inputs);
    }
}
