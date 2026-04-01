use std::{collections::BTreeMap, fmt::Display, sync::{Arc, Mutex}};

pub type JobTable = Arc<Mutex<Jobs>>;

#[derive(Debug, Default)]
pub struct JobInfo {
    pub complete: bool,
    pub command_string: String,
}

#[derive(Debug, Default)]
pub struct Jobs {
    pub counter: usize,
    pub job_table: BTreeMap<usize, JobInfo>,
}

impl Jobs {
    pub fn insert_job(&mut self, command_string: String) -> usize {
        self.counter += 1;
        self.job_table.insert(
            self.counter,
            JobInfo {
                complete: false,
                command_string,
            },
        );
        self.counter
    }

    pub fn set_job_complete(&mut self, number: usize) {
        self.job_table.get_mut(&number).unwrap().complete = true;
    }

    pub fn clean_completed_jobs(&mut self) {
        self.job_table.retain(|_, info| !info.complete);
    }
}

impl Display for Jobs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let job_count = self.job_table.len();
        for (index, (number, info)) in self.job_table.iter().enumerate() {
            let marker = if index + 2 == job_count {
                "-"
            } else if index + 1 == job_count {
                "+"
            } else {
                " "
            };
            let status = if info.complete { "Done" } else { "Running" };

            writeln!(f, "[{number}]{marker}  {status:<24}{}", info.command_string)?;
        }

        Ok(())
    }
}
