use indicatif::ProgressIterator;
use std::collections::HashMap;

pub type Distance = f32;
pub type Index = u16;

pub struct NameGraph {
    distances: HashMap<(Index, Index), Distance>,
    name_count: usize,
}

impl NameGraph {
    pub fn new(len: usize) -> NameGraph {
        NameGraph {
            distances: HashMap::new(),
            name_count: len,
        }
    }

    pub fn fill(&mut self, names: Vec<String>) {
        // Init progressbar
        let progress = indicatif::ProgressBar::new(self.name_count as u64 - 1);
        progress.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("Computing Levenshtein distances {percent}% {wide_bar} [🕒 {elapsed_precise} ETA {eta_precise}]"),
        );

        for i in (0..self.name_count - 1).progress_with(progress) {
            for j in (i + 1)..self.name_count {
                let dist = strsim::normalized_levenshtein(&names[i], &names[j]);
                self.distances.insert((i as Index, j as Index), dist as f32);
                log::trace!("#{} {:?} - #{} {:?} = {}", i, names[i], j, names[j], dist);
            }
        }
    }

    pub fn get_distance(&self, a: Index, b: Index) -> Distance {
        let key = if a <= b { (a, b) } else { (b, a) };
        *self.distances.get(&key).unwrap()
    }
}
