use indicatif::ProgressIterator;

pub type Distance = f32;

pub struct NameGraph {
    distances: Vec<Vec<Distance>>,
}

impl NameGraph {
    pub fn new() -> NameGraph {
        NameGraph {
            distances: Vec::new(),
        }
    }

    pub fn fill(&mut self, names: &[String]) {
        // Init progressbar
        let progress = indicatif::ProgressBar::new(names.len() as u64);
        progress.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("Computing Levenshtein distances {percent}% {wide_bar} [🕒 {elapsed_precise} ETA {eta_precise}]"),
        );
        progress.set_draw_delta(names.len() as u64 / 100);

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
                        cur_vec.reserve(names.len() - i - 1);
                        for j in i + 1..names.len() {
                            cur_vec.push(Self::compute_distance(i, j, names));
                        }
                        vec_tx.send((i, cur_vec)).unwrap();
                    }
                });
            }

            // Send indexes and get vector from threads
            for i in 0..names.len() - 1 {
                work_tx.send(i).unwrap();
            }
            self.distances.resize(names.len(), Vec::new());
            for _ in (0..names.len() - 1).progress_with(progress) {
                let (i, v) = vec_rx.recv().unwrap();
                self.distances[i] = v;
            }
        })
        .unwrap();
    }

    #[allow(dead_code)]
    pub fn get_distance(&self, a: usize, b: usize) -> Distance {
        match a.cmp(&b) {
            std::cmp::Ordering::Equal => 0.0,
            std::cmp::Ordering::Less => self.distances[a][b - a - 1],
            std::cmp::Ordering::Greater => self.distances[b][a - b - 1],
        }
    }

    fn compute_distance(a: usize, b: usize, names: &[String]) -> Distance {
        let name_a = &names[a];
        let name_b = &names[b];
        let dist = strsim::normalized_levenshtein(name_a, name_b);
        log::trace!("#{} {:?} - #{} {:?} = {}", a, name_a, b, name_b, dist);
        1.0 - dist as Distance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_distance() {
        let names = ["John".to_string(), "Johnny".to_string(), "Bob".to_string()];
        assert_eq!(NameGraph::compute_distance(0, 0, &names), 0.0);
        assert_eq!(NameGraph::compute_distance(0, 1, &names), 0.3333333);
        assert_eq!(NameGraph::compute_distance(0, 2, &names), 0.75);
        assert_eq!(NameGraph::compute_distance(1, 0, &names), 0.3333333);
        assert_eq!(NameGraph::compute_distance(1, 1, &names), 0.0);
        assert_eq!(NameGraph::compute_distance(1, 2, &names), 0.8333333);
        assert_eq!(NameGraph::compute_distance(2, 0, &names), 0.75);
        assert_eq!(NameGraph::compute_distance(2, 1, &names), 0.8333333);
        assert_eq!(NameGraph::compute_distance(2, 2, &names), 0.0);
    }

    #[test]
    fn test_get_distance() {
        let mut graph = NameGraph::new();
        let names = ["John".to_string(), "Johnny".to_string(), "Bob".to_string()];
        graph.fill(&names);
        assert_eq!(graph.get_distance(0, 0), 0.0);
        assert_eq!(graph.get_distance(0, 1), 0.3333333);
        assert_eq!(graph.get_distance(0, 2), 0.75);
        assert_eq!(graph.get_distance(1, 0), 0.3333333);
        assert_eq!(graph.get_distance(1, 1), 0.0);
        assert_eq!(graph.get_distance(1, 2), 0.8333333);
        assert_eq!(graph.get_distance(2, 0), 0.75);
        assert_eq!(graph.get_distance(2, 1), 0.8333333);
        assert_eq!(graph.get_distance(2, 2), 0.0);
    }
}
