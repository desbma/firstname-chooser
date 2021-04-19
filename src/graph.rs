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
            // Setup channel & thread pool to compute distances
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
        if a == b {
            0.0
        } else if a <= b {
            self.distances[b][b - a - 1]
        } else {
            self.distances[a][a - b as usize - 1]
        }
    }

    fn compute_distance(a: usize, b: usize, names: &[String]) -> Distance {
        let name_a = &names[a];
        let name_b = &names[b];
        let dist = strsim::normalized_levenshtein(name_a, name_b);
        log::trace!("#{} {:?} - #{} {:?} = {}", a, name_a, b, name_b, dist);
        dist as f32
    }
}
