use indicatif::ProgressIterator;
use std::collections::HashMap;

pub type Distance = f32;
pub type Index = u16;

pub struct NameGraph {
    distances: HashMap<(Index, Index), Distance>, // TODO use 2 hashmaps or BTreeMap instead
    name_count: usize,
}

impl NameGraph {
    pub fn new(len: usize) -> NameGraph {
        NameGraph {
            distances: HashMap::new(),
            name_count: len,
        }
    }

    pub fn fill(&mut self, names: &[String]) {
        // Init progressbar
        let dist_count = IndexIterator::new(self.name_count).len();
        debug_assert_eq!(dist_count, IndexIterator::new(self.name_count).count());
        let progress = indicatif::ProgressBar::new(dist_count as u64);
        progress.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("Computing Levenshtein distances {percent}% {wide_bar} [🕒 {elapsed_precise} ETA {eta_precise}]"),
        );
        progress.set_draw_delta(dist_count as u64 / 100);

        // Compute distances
        self.distances.reserve(dist_count);
        self.distances.extend(
            IndexIterator::new(self.name_count)
                .progress_with(progress)
                .map(|(i, j)| {
                    (
                        (i as Index, j as Index),
                        Self::compute_distance(i, j, &names),
                    )
                }),
        );
    }

    #[allow(dead_code)]
    pub fn get_distance(&self, a: Index, b: Index) -> Distance {
        let key = if a <= b { (a, b) } else { (b, a) };
        *self.distances.get(&key).unwrap()
    }

    fn compute_distance(a: usize, b: usize, names: &[String]) -> Distance {
        let name_a = &names[a];
        let name_b = &names[b];
        let dist = strsim::normalized_levenshtein(name_a, name_b);
        log::trace!("#{} {:?} - #{} {:?} = {}", a, name_a, b, name_b, dist);
        dist as f32
    }
}

#[derive(Clone)]
struct IndexIterator {
    i: usize,
    j: usize,
    len: usize,
}

impl IndexIterator {
    pub fn new(len: usize) -> IndexIterator {
        IndexIterator { i: 0, j: 1, len }
    }
}

impl Iterator for IndexIterator {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        while self.i < self.len - 1 {
            if self.j < self.len {
                let cur_j = self.j;
                self.j += 1;
                return Some((self.i, cur_j));
            }
            self.i += 1;
            self.j = self.i + 1;
        }
        None
    }
}

impl ExactSizeIterator for IndexIterator {
    fn len(&self) -> usize {
        let mut r = 0;
        let mut j = self.j;
        for i in self.i..(self.len - 1) {
            r += self.len - j;
            j = i + 2;
        }
        r
    }
}
