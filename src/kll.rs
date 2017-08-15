use rand::random;

use std::vec::Vec;
use std::marker::PhantomData;
use std::iter::Sum;

pub trait Compact<T: Ord> {
    fn compact(&mut self) -> Option<T>;
}

impl<T> Compact<T> for Vec<T>
    where T: Ord
{
    fn compact(&mut self) -> Option<T> {
        self.sort();

        if self.len() < 2 {
            return None;
        }

        if random() {
            self.pop();
            return self.pop();
        } else {
            let a = self.pop();
            self.pop();
            return a;
        }
    }
}

pub struct HLL<T: Ord> {
    k: usize,
    c: f64,
    compactors: Vec<Vec<T>>,
    size: usize,
    max_size: usize,
    mk: PhantomData<T>,
}

impl<'a, T> HLL<T>
    where T: Ord + Clone + Sum<usize>
{
    pub fn new(k: usize, c: f64) -> HLL<T> {
        let mut a = HLL {
            k: k,
            c: c,
            compactors: Vec::<Vec<T>>::new(),
            max_size: 0,
            size: 0,
            mk: PhantomData,
        };
        a.grow();
        a
    }

    fn grow(&mut self) {
        self.compactors.push(Vec::<T>::new());
        let n_compactors = self.compactors.len();
        self.max_size = (0..n_compactors)
            .map(|height| {
                     let depth = n_compactors - height - 1;
                     (self.c.powf(depth as f64) * self.k as f64) as usize + 1
                 })
            .sum()
    }

    fn capacity(&self, height: usize) -> usize {
        let n_compactors = self.compactors.len();
        let depth = n_compactors - height - 1;
        (self.c.powf(depth as f64) * self.k as f64) as usize + 1
    }

    pub fn update(&mut self, item: T) {
        self.compactors[0].push(item);
        self.size += 1;
        if self.size >= self.max_size {
            self.compress();
        }
    }

    pub fn compress(&mut self) {
        let n_compactors = self.compactors.len();
        for h in 0..n_compactors {
            if self.compactors[h].len() >= self.capacity(h) {
                if h + 1 >= n_compactors {
                    self.grow();
                }
                if let Some(val) = self.compactors[h].compact() {
                    self.compactors[h + 1].push(val);
                }
                break;
            }
        }
    }

    pub fn merge(&mut self, other: &Self) {
        while self.compactors.len() < other.compactors.len() {
            self.grow();
        }

        for h in 0..other.compactors.len() {
            let other_compactor = other.compactors[h].clone();
            self.compactors[h].extend(other_compactor);
        }

        self.size = self.compactors.iter().map(|c| c.len()).sum();
        while self.size >= self.max_size {
            self.compress();
        }
    }

    pub fn rank(&self, value: &T) -> usize {
        self.compactors
            .iter()
            .enumerate()
            .map(|(h, c)| {
                c.iter()
                    .map(|item| if item <= value {
                             2usize.pow(h as u32)
                         } else {
                             0
                         })
                    .sum::<usize>()
            })
            .sum::<usize>()
    }

    pub fn cdf(&self) -> Vec<(T, f64)> {
        let mut items_and_weights: Vec<(&T, usize)> = self.compactors
            .iter()
            .enumerate()
            .flat_map(|(h, c)| {
                          c.iter()
                              .map(|item| (item, (2 as usize).pow(h as u32)))
                              .collect::<Vec<(&T, usize)>>()
                      })
            .collect();
        let total_weight = items_and_weights.iter().map(|&(_, w)| w).sum::<usize>();
        items_and_weights.sort();
        let mut cum_weight: usize = 0;
        items_and_weights
            .iter()
            .map(|&(item, weight)| {
                     cum_weight += weight; // janky af
                     (item.clone(), cum_weight as f64 / total_weight as f64)
                 })
            .collect()
    }

    pub fn ranks(&self) -> Vec<(T, usize)> {
        let mut items_and_weights: Vec<(&T, usize)> = self.compactors
            .iter()
            .enumerate()
            .flat_map(|(h, c)| {
                          c.iter()
                              .map(|item| (item, 2usize.pow(h as u32)))
                              .collect::<Vec<(&T, usize)>>()
                      })
            .collect();
        items_and_weights.sort();
        let mut cum_weight = 0;
        items_and_weights
            .iter()
            .map(|&(item, weight)| {
                     cum_weight += weight;
                     (item.clone(), cum_weight)
                 })
            .collect()
    }
}
