//! A tiny pet that feeds on commits.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Happy,
    Hungry,
    Sleeping,
}

impl fmt::Display for Mood {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Mood::Happy => "happy",
            Mood::Hungry => "hungry",
            Mood::Sleeping => "sleeping",
        };
        write!(f, "{s}")
    }
}

/// A pet whose hunger always stays in 0..=100.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pet {
    hunger: u8,
}

impl Default for Pet {
    fn default() -> Self {
        Pet { hunger: 50 }
    }
}

const MAX_HUNGER: u8 = 100;
const STEP: u8 = 10;

impl Pet {
    pub fn hunger(&self) -> u8 {
        self.hunger
    }

    /// Each commit lowers hunger by 10, never below 0.
    pub fn feed(&mut self, commits: usize) {
        let drop = u8::try_from(commits.saturating_mul(usize::from(STEP))).unwrap_or(u8::MAX);
        self.hunger = self.hunger.saturating_sub(drop);
    }

    /// Each hour raises hunger by 10, never above 100.
    pub fn starve(&mut self, hours: u32) {
        let add = u8::try_from(hours.saturating_mul(u32::from(STEP))).unwrap_or(u8::MAX);
        self.hunger = self.hunger.saturating_add(add).min(MAX_HUNGER);
    }

    pub fn mood(&self) -> Mood {
        if self.hunger < 30 {
            Mood::Happy
        } else if self.hunger > 70 {
            Mood::Hungry
        } else {
            Mood::Sleeping
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feeding_never_underflows() {
        let mut p = Pet::default();
        p.feed(1000);
        assert_eq!(p.hunger(), 0);
        assert_eq!(p.mood(), Mood::Happy);
    }
}
