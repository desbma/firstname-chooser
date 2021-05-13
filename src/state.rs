use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::iter::IntoIterator;

use serde::{Deserialize, Serialize};

pub struct Choice {
    pub name: String,
    pub index: usize,
    pub liked: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct StoredChoice {
    name: String,
    liked: bool,
}

pub struct State {
    choices: VecDeque<Choice>,
    csv_writer: csv::Writer<File>,
}

impl State {
    pub fn new(names: &[String]) -> anyhow::Result<State> {
        let binary_name = env!("CARGO_PKG_NAME");
        let xdg_dirs = xdg::BaseDirectories::with_prefix(binary_name)?;
        let csv_filepath = xdg_dirs.place_data_file("state.csv")?;
        let csv_file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(&csv_filepath)?;
        let file_reader = BufReader::new(&csv_file);
        let mut csv_reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(file_reader);

        let mut choices = VecDeque::new();
        for result in csv_reader.deserialize() {
            let record: StoredChoice = result?;
            let index = match names.iter().position(|n| n == &record.name) {
                Some(i) => i,
                None => {
                    log::warn!("Unable to find index of previous choice {:?}", record.name);
                    continue;
                }
            };
            choices.push_back(Choice {
                name: record.name,
                index,
                liked: record.liked,
            });
        }

        log::info!(
            "Loaded {} previous choices from {:?}",
            choices.len(),
            csv_filepath
        );

        let csv_writer = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(csv_file);

        Ok(State {
            choices,
            csv_writer,
        })
    }

    pub fn save(&mut self, name: &str, index: usize, liked: bool) -> anyhow::Result<()> {
        self.choices.push_back(Choice {
            name: name.to_string(),
            index,
            liked,
        });
        self.csv_writer.serialize(StoredChoice {
            name: name.to_string(),
            liked,
        })?;
        self.csv_writer.flush()?;
        Ok(())
    }
}

impl<'a> IntoIterator for &'a State {
    type Item = &'a Choice;
    type IntoIter = std::collections::vec_deque::Iter<'a, Choice>;

    fn into_iter(self) -> Self::IntoIter {
        self.choices.iter()
    }
}
