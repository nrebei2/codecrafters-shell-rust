use std::{
    fs,
    io::{BufRead, BufReader, BufWriter, Write},
    ops::{AddAssign, Deref},
    path::PathBuf,
};

#[derive(Default)]
pub struct History {
    inputs: Vec<String>,
    last_append_index: usize,
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
            last_append_index: 0,
        })
    }

    pub fn write_to_file(&mut self, file_path: PathBuf, append: bool) -> std::io::Result<()> {
        let mut file = BufWriter::new(
            fs::OpenOptions::new()
                .create(true)
                .truncate(!append)
                .write(true)
                .append(append)
                .open(file_path)?,
        );

        if append {
            for input in &self.inputs[self.last_append_index..] {
                writeln!(file, "{input}")?;
            }
            self.last_append_index = self.inputs.len();
        } else {
            for input in &self.inputs {
                writeln!(file, "{input}")?;
            }
        }

        Ok(())
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
