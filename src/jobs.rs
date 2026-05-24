use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    fmt::Display,
    sync::{Arc, Mutex},
};

use indexmap::IndexMap;

pub type JobTable = Arc<Mutex<Jobs>>;

#[derive(Debug, Default)]
pub struct JobInfo {
    pub complete: bool,
    pub command_string: String,
    marker: &'static str,
}

#[derive(Debug, Default)]
pub struct Jobs {
    job_table: IndexMap<usize, JobInfo>,
    free_job_numbers: BinaryHeap<Reverse<usize>>,
}

impl Jobs {
    pub fn insert_job(&mut self, command_string: String) -> usize {
        let next_job_number = self
            .free_job_numbers
            .pop()
            .map(|r| r.0)
            .unwrap_or(self.job_table.len() + 1);

        let mut job_iter = self.job_table.values_mut().rev();

        if let Some(job1) = job_iter.next() {
            job1.marker = "-";
        }

        if let Some(job2) = job_iter.next() {
            job2.marker = "";
        }

        self.job_table.insert(
            next_job_number,
            JobInfo {
                complete: false,
                command_string,
                marker: "+",
            },
        );
        next_job_number
    }

    pub fn set_job_complete(&mut self, number: usize) {
        self.job_table.get_mut(&number).unwrap().complete = true;
    }

    pub fn clean_completed_jobs(&mut self, print: bool) {
        self.job_table.retain(|&job_number, info| {
            if info.complete {
                if print {
                    let status = if info.complete { "Done" } else { "Running" };
                    println!(
                        "[{job_number}]{}  {status:<24}{}",
                        info.marker,
                        if info.complete {
                            info.command_string
                                .trim_end_matches(|c: char| c.is_ascii_whitespace() || c == '&')
                        } else {
                            &info.command_string
                        }
                    );
                }
                self.free_job_numbers.push(Reverse(job_number));
                false
            } else {
                true
            }
        });
    }
}

impl Display for Jobs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (number, info) in self.job_table.iter() {
            let status = if info.complete { "Done" } else { "Running" };
            writeln!(
                f,
                "[{number}]{}  {status:<24}{}",
                info.marker,
                if info.complete {
                    info.command_string
                        .trim_end_matches(|c: char| c.is_ascii_whitespace() || c == '&')
                } else {
                    &info.command_string
                }
            )?;
        }

        Ok(())
    }
}
