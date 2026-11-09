//! Quiz scoring and item ordering. Pure: no I/O.

use super::curriculum::Item;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Score {
    pub correct: usize,
    pub total: usize,
    pub percent: u8,
    pub passed: bool,
    /// Indices into the item list of the items answered wrongly.
    pub missed: Vec<usize>,
}

/// Score `answers[i]` against `items[i]`; a missing answer counts as wrong.
pub fn score(items: &[Item], answers: &[usize]) -> Score {
    let missed: Vec<usize> = (0..items.len())
        .filter(|&i| answers.get(i) != Some(&items[i].answer))
        .collect();
    let total = items.len();
    let correct = total - missed.len();
    let percent = (correct * 100).checked_div(total).unwrap_or(0);
    Score {
        correct,
        total,
        percent: percent as u8,
        passed: total > 0 && correct * 5 >= total * 4,
        missed,
    }
}

/// A shuffled order of `0..n`; never equal to `prev` when `n >= 2`.
pub fn order(n: usize, prev: Option<&[usize]>, seed: u64) -> Vec<usize> {
    let mut x = seed ^ 0x9E37_79B9_7F4A_7C15;
    if x == 0 {
        x = 1;
    }
    let mut v: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        v.swap(i, (x % (i as u64 + 1)) as usize);
    }
    if n >= 2 && prev == Some(v.as_slice()) {
        v.rotate_left(1);
    }
    v
}

/// A seed from the clock's nanoseconds.
pub fn clock_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::academy::curriculum::{ItemKind, ObjId};

    fn items(n: usize) -> Vec<Item> {
        (0..n)
            .map(|_| Item {
                kind: ItemKind::MultipleChoice,
                prompt: "p",
                choices: &["a", "b"],
                answer: 1,
                tags: &[ObjId(1, 1)],
            })
            .collect()
    }

    #[test]
    fn lrn_18_a_eight_of_ten_passes_seven_fails() {
        let it = items(10);
        let mut a = vec![1; 10];
        a[0] = 0;
        a[1] = 0;
        let s = score(&it, &a);
        assert_eq!((s.correct, s.percent, s.passed), (8, 80, true));
        a[2] = 0;
        let s = score(&it, &a);
        assert_eq!((s.correct, s.percent, s.passed), (7, 70, false));
    }

    #[test]
    fn score_reports_missed_items() {
        let s = score(&items(4), &[1, 0, 1]);
        assert_eq!(s.missed, vec![1, 3]);
        assert_eq!(s.total, 4);
    }

    #[test]
    fn lrn_20_a_retake_order_differs() {
        for n in [2usize, 3, 10] {
            for seed in 0..1000u64 {
                let prev = order(n, None, seed.wrapping_mul(31));
                let next = order(n, Some(&prev), seed);
                assert_ne!(next, prev, "n={n} seed={seed}");
                let mut sorted = next.clone();
                sorted.sort();
                assert_eq!(sorted, (0..n).collect::<Vec<_>>());
            }
        }
    }
}
