use indicatif::ProgressIterator;

pub type Distance = f32;
pub type Index = u16;

pub struct NameGraph {
    distances: Vec<Vec<Distance>>,
    name_count: usize,
}

impl NameGraph {
    pub fn new(len: usize) -> NameGraph {
        NameGraph {
            distances: Vec::new(),
            name_count: len,
        }
    }

    pub fn fill(&mut self, names: &[String]) {
        // Init progressbar
        let progress = indicatif::ProgressBar::new(self.name_count as u64);
        progress.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("Computing Levenshtein distances {percent}% {wide_bar} [🕒 {elapsed_precise} ETA {eta_precise}]"),
        );
        progress.set_draw_delta(self.name_count as u64 / 100);

        // Compute distances
        self.distances.reserve(self.name_count - 1);
        for i in (0..self.name_count - 1).progress_with(progress) {
            let mut cur_vec: Vec<Distance> = Vec::new();
            cur_vec.reserve(self.name_count - i - 1);
            for j in i + 1..self.name_count {
                cur_vec.push(Self::compute_distance(i, j, names));
            }
            self.distances.push(cur_vec);
        }
    }

    #[allow(dead_code)]
    pub fn get_distance(&self, a: Index, b: Index) -> Distance {
        if a == b {
            0.0
        } else if a <= b {
            self.distances[b as usize][b as usize - a as usize - 1]
        } else {
            self.distances[a as usize][a as usize - b as usize - 1]
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
