use std::cmp::Ordering;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Distance {
    Euclidean,
    Cosine,
    DotProduct,
}

pub fn get_distance_fn(distance: Distance) -> fn(&[f32], &[f32]) -> f32 {
    match distance {
        Distance::Euclidean => euclidean,
        Distance::Cosine => cosine,
        Distance::DotProduct => dot_product,
    }
}

fn euclidean(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).fold(0.0, |acc, (x, y)| {
        let diff = x - y;
        acc + diff * diff
    }).sqrt()
}

fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).fold(0.0, |acc, (x, y)| acc + x * y)
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let a = normalize(a);
    let b = normalize(b);
    a.iter().zip(b).fold(0.0, |acc, (x, y)| acc + x * y)
}

pub fn normalize(vec: &[f32]) -> Vec<f32> {
    let magnitude = (vec.iter().fold(0.0, |acc, &val| val.mul_add(val, acc))).sqrt();

    if magnitude > std::f32::EPSILON {
        vec.iter().map(|&val| val / magnitude).collect()
    } else {
        vec.to_vec()
    }
}

#[derive(Debug, Clone)]
pub struct ScoreIndex {
    pub score: f32,
    pub index: usize,
}

impl PartialEq for ScoreIndex {
    fn eq(&self, other: &Self) -> bool {
        self.score.eq(&other.score)
    }
}

impl Eq for ScoreIndex {}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for ScoreIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // The comparison is intentionally reversed here to make the heap a min-heap
        other.score.partial_cmp(&self.score)
    }
}

impl Ord for ScoreIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}
