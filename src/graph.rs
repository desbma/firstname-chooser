use indicatif::ProgressIterator;
use rand::Rng;
use std::collections::HashMap;

pub type Distance = decorum::R64; // f64 with total ordering

pub struct LevenshteinGraph {
    distances: Vec<Vec<Distance>>,
}

impl LevenshteinGraph {
    pub fn new() -> LevenshteinGraph {
        LevenshteinGraph {
            distances: Vec::new(),
        }
    }

    pub fn fill(&mut self, words: &[String]) {
        // Init progressbar
        let progress = indicatif::ProgressBar::new(words.len() as u64);
        progress.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("Computing Levenshtein distances {percent}% {wide_bar} [🕒 {elapsed_precise} ETA {eta_precise}]"),
        );
        progress.set_draw_delta(words.len() as u64 / 100);

        crossbeam::thread::scope(|scope| {
            // Setup channel & threads to compute distances
            let (work_tx, work_rx) = crossbeam::channel::unbounded();
            let (vec_tx, vec_rx) = crossbeam::channel::unbounded();
            for _ in 0..num_cpus::get() {
                let work_rx = work_rx.clone();
                let vec_tx = vec_tx.clone();
                scope.spawn(move |_| {
                    while let Ok(i) = work_rx.recv() {
                        let mut cur_vec: Vec<Distance> = Vec::new();
                        cur_vec.reserve(words.len() - i - 1);
                        for j in i + 1..words.len() {
                            cur_vec.push(Self::compute_distance(i, j, words));
                        }
                        vec_tx.send((i, cur_vec)).unwrap();
                    }
                });
            }

            // Send indexes and get vector from threads
            for i in 0..words.len() - 1 {
                work_tx.send(i).unwrap();
            }
            self.distances.resize(words.len(), Vec::new());
            for _ in (0..words.len() - 1).progress_with(progress) {
                let (i, v) = vec_rx.recv().unwrap();
                self.distances[i] = v;
            }
        })
        .unwrap();
    }

    pub fn get_distance(&self, a: usize, b: usize) -> Distance {
        match a.cmp(&b) {
            std::cmp::Ordering::Equal => 0.0.into(),
            std::cmp::Ordering::Less => self.distances[a][b - a - 1],
            std::cmp::Ordering::Greater => self.distances[b][a - b - 1],
        }
    }

    fn compute_distance(a: usize, b: usize, words: &[String]) -> Distance {
        let word_a = &words[a];
        let word_b = &words[b];
        let dist = strsim::normalized_levenshtein(word_a, word_b);
        log::trace!("#{} {:?} - #{} {:?} = {}", a, word_a, b, word_b, dist);
        (1.0 - dist).into()
    }

    pub fn random(&self) -> usize {
        let mut rng = rand::thread_rng();
        rng.gen_range(0..self.distances.len())
    }

    fn evaluate(&self, idx: usize, prev_choices: &HashMap<usize, bool>) -> Distance {
        let mut v: Distance = 0.0.into();
        for prev_choice in prev_choices {
            let dist = self.get_distance(idx, *prev_choice.0);
            v += match prev_choice.1 {
                true => dist,
                false => -dist,
            };
        }
        v
    }

    pub fn recommend(&self, prev_choices: &HashMap<usize, bool>) -> usize {
        self.distances
            .iter()
            .enumerate()
            .filter(|(i, _)| !prev_choices.contains_key(i)) // exclude already evaluated choices
            .min_by_key(|(i, _)| self.evaluate(*i, prev_choices))
            .unwrap()
            .0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_distance() {
        let words = ["John".to_string(), "Johnny".to_string(), "Bob".to_string()];
        assert_eq!(
            LevenshteinGraph::compute_distance(0, 0, &words).into_inner(),
            0.0
        );
        assert_eq!(
            LevenshteinGraph::compute_distance(0, 1, &words).into_inner(),
            0.33333333333333326
        );
        assert_eq!(
            LevenshteinGraph::compute_distance(0, 2, &words).into_inner(),
            0.75
        );
        assert_eq!(
            LevenshteinGraph::compute_distance(1, 0, &words).into_inner(),
            0.33333333333333326
        );
        assert_eq!(
            LevenshteinGraph::compute_distance(1, 1, &words).into_inner(),
            0.0
        );
        assert_eq!(
            LevenshteinGraph::compute_distance(1, 2, &words).into_inner(),
            0.8333333333333334
        );
        assert_eq!(
            LevenshteinGraph::compute_distance(2, 0, &words).into_inner(),
            0.75
        );
        assert_eq!(
            LevenshteinGraph::compute_distance(2, 1, &words).into_inner(),
            0.8333333333333334
        );
        assert_eq!(
            LevenshteinGraph::compute_distance(2, 2, &words).into_inner(),
            0.0
        );
    }

    #[test]
    fn test_get_distance() {
        let mut graph = LevenshteinGraph::new();
        let words = ["John".to_string(), "Johnny".to_string(), "Bob".to_string()];
        graph.fill(&words);
        assert_eq!(graph.get_distance(0, 0).into_inner(), 0.0);
        assert_eq!(graph.get_distance(0, 1).into_inner(), 0.33333333333333326);
        assert_eq!(graph.get_distance(0, 2).into_inner(), 0.75);
        assert_eq!(graph.get_distance(1, 0).into_inner(), 0.33333333333333326);
        assert_eq!(graph.get_distance(1, 1).into_inner(), 0.0);
        assert_eq!(graph.get_distance(1, 2).into_inner(), 0.8333333333333334);
        assert_eq!(graph.get_distance(2, 0).into_inner(), 0.75);
        assert_eq!(graph.get_distance(2, 1).into_inner(), 0.8333333333333334);
        assert_eq!(graph.get_distance(2, 2).into_inner(), 0.0);
    }
}
