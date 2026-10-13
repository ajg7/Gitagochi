//! Pure pet state and rules. No terminal, git or filesystem I/O.

use serde::{Deserialize, Serialize};

/// Hunger gained per second (0..=100 scale): empty to starving in ~8 hours.
pub const HUNGER_PER_SEC: f64 = 100.0 / (8.0 * 3600.0);
/// Hunger at or above which the pet is hungry and loses health.
pub const HUNGRY_THRESHOLD: f64 = 60.0;
/// Hunger at or above which the pet is sleepy.
pub const SLEEPY_HUNGER: f64 = 90.0;
/// Health at or below which the pet is sleepy.
pub const SLEEPY_HEALTH: f64 = 25.0;
/// Health lost per second while hungry: full to empty in ~12 hours.
pub const HEALTH_DECAY_PER_SEC: f64 = 100.0 / (12.0 * 3600.0);
/// Hunger removed per commit fed.
pub const HUNGER_PER_COMMIT: f64 = 25.0;
/// Health restored per commit fed.
pub const HEALTH_PER_COMMIT: f64 = 5.0;
/// Commits needed to leave the egg stage.
pub const BABY_AT: u64 = 3;
/// Commits needed to become an adult.
pub const ADULT_AT: u64 = 20;
/// How many recently fed commits are remembered.
pub const MAX_RECENT: usize = 20;
/// How many fed commit ids are remembered to avoid feeding twice.
pub const MAX_FED_IDS: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FedCommit {
    pub id: String,
    pub summary: String,
    pub time: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Egg,
    Baby,
    Adult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Happy,
    Content,
    Hungry,
    Sleepy,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Pet {
    pub hunger: f64,
    pub health: f64,
    pub total_fed: u64,
    /// Unix seconds of the last time decay was applied.
    pub last_update: i64,
    /// True once the repo has been baselined (first run done).
    pub initialized: bool,
    /// HEAD commit id at the last check; `None` for an empty repo.
    pub last_seen: Option<String>,
    /// Most recent fed commits, newest first.
    pub recent: Vec<FedCommit>,
    /// Ids of every fed commit (oldest first, bounded by `MAX_FED_IDS`).
    pub fed_ids: Vec<String>,
}

impl Default for Pet {
    fn default() -> Self {
        Pet::new(0)
    }
}

impl Pet {
    pub fn new(now: i64) -> Self {
        Pet {
            hunger: 50.0,
            health: 100.0,
            total_fed: 0,
            last_update: now,
            initialized: false,
            last_seen: None,
            recent: Vec::new(),
            fed_ids: Vec::new(),
        }
    }

    /// Clamp values that may be garbage (e.g. hand-edited or NaN save files).
    pub fn sanitize(&mut self) {
        let fix = |v: f64| if v.is_nan() { 0.0 } else { v.clamp(0.0, 100.0) };
        self.hunger = fix(self.hunger);
        self.health = fix(self.health);
        self.recent.truncate(MAX_RECENT);
        for r in self.recent.iter().rev() {
            if !self.fed_ids.contains(&r.id) {
                self.fed_ids.push(r.id.clone());
            }
        }
        self.trim_fed_ids();
    }

    fn trim_fed_ids(&mut self) {
        if self.fed_ids.len() > MAX_FED_IDS {
            let extra = self.fed_ids.len() - MAX_FED_IDS;
            self.fed_ids.drain(..extra);
        }
    }

    pub fn has_fed(&self, id: &str) -> bool {
        self.fed_ids.iter().any(|x| x == id)
    }

    /// Apply time-based decay up to `now` (unix seconds). If the clock went
    /// backwards, no decay is applied and the reference time is reset.
    pub fn tick(&mut self, now: i64) {
        let elapsed = now.saturating_sub(self.last_update);
        self.last_update = now;
        if elapsed <= 0 {
            return;
        }
        let elapsed = elapsed as f64;
        let h0 = self.hunger;
        let h1 = (h0 + elapsed * HUNGER_PER_SEC).min(100.0);
        // Time spent at or above the hungry threshold during this interval.
        let secs_to_threshold = if h0 >= HUNGRY_THRESHOLD {
            0.0
        } else {
            (HUNGRY_THRESHOLD - h0) / HUNGER_PER_SEC
        };
        let hungry_secs = (elapsed - secs_to_threshold).max(0.0);
        self.hunger = h1;
        self.health = (self.health - hungry_secs * HEALTH_DECAY_PER_SEC).max(0.0);
    }

    /// Feed the pet with the given commits (any order; stored newest first
    /// as given by the caller, who should pass newest first).
    pub fn feed(&mut self, commits: &[FedCommit]) {
        for c in commits.iter().rev() {
            self.hunger = (self.hunger - HUNGER_PER_COMMIT).max(0.0);
            self.health = (self.health + HEALTH_PER_COMMIT).min(100.0);
            self.total_fed = self.total_fed.saturating_add(1);
            self.recent.insert(0, c.clone());
            self.fed_ids.push(c.id.clone());
        }
        self.recent.truncate(MAX_RECENT);
        self.trim_fed_ids();
    }

    pub fn stage(&self) -> Stage {
        if self.total_fed >= ADULT_AT {
            Stage::Adult
        } else if self.total_fed >= BABY_AT {
            Stage::Baby
        } else {
            Stage::Egg
        }
    }

    pub fn mood(&self) -> Mood {
        if self.hunger >= SLEEPY_HUNGER || self.health <= SLEEPY_HEALTH {
            Mood::Sleepy
        } else if self.hunger >= HUNGRY_THRESHOLD {
            Mood::Hungry
        } else if self.hunger < 30.0 && self.health >= 70.0 {
            Mood::Happy
        } else {
            Mood::Content
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commit(n: u32) -> FedCommit {
        FedCommit {
            id: format!("id{n}"),
            summary: format!("commit {n}"),
            time: 0,
        }
    }

    #[test]
    fn mood_thresholds() {
        let mut p = Pet::new(0);
        p.hunger = 10.0;
        p.health = 100.0;
        assert_eq!(p.mood(), Mood::Happy);
        p.hunger = 40.0;
        assert_eq!(p.mood(), Mood::Content);
        p.hunger = 60.0;
        assert_eq!(p.mood(), Mood::Hungry);
        p.hunger = 90.0;
        assert_eq!(p.mood(), Mood::Sleepy);
        p.hunger = 10.0;
        p.health = 25.0;
        assert_eq!(p.mood(), Mood::Sleepy);
    }

    #[test]
    fn hunger_rises_with_time() {
        let mut p = Pet::new(0);
        p.hunger = 0.0;
        p.tick(3600);
        assert!((p.hunger - 3600.0 * HUNGER_PER_SEC).abs() < 1e-9);
        assert_eq!(p.health, 100.0);
    }

    #[test]
    fn hunger_caps_and_health_decays_but_never_below_zero() {
        let mut p = Pet::new(0);
        p.tick(10 * 24 * 3600);
        assert_eq!(p.hunger, 100.0);
        assert_eq!(p.health, 0.0);
        assert_eq!(p.mood(), Mood::Sleepy);
    }

    #[test]
    fn health_only_decays_after_threshold() {
        let mut p = Pet::new(0);
        p.hunger = 0.0;
        let to_threshold = (HUNGRY_THRESHOLD / HUNGER_PER_SEC) as i64;
        p.tick(to_threshold - 10);
        assert_eq!(p.health, 100.0);
        p.tick(to_threshold + 3600);
        assert!(p.health < 100.0);
    }

    #[test]
    fn feeding_lowers_hunger_and_counts() {
        let mut p = Pet::new(0);
        p.hunger = 80.0;
        p.feed(&[commit(1), commit(2)]);
        assert_eq!(p.hunger, 30.0);
        assert_eq!(p.total_fed, 2);
        assert_eq!(p.recent[0].id, "id1");
        p.feed(&[commit(3), commit(4), commit(5)]);
        assert_eq!(p.hunger, 0.0);
        assert_eq!(p.health, 100.0);
    }

    #[test]
    fn recent_list_is_bounded() {
        let mut p = Pet::new(0);
        let cs: Vec<_> = (0..100).map(commit).collect();
        p.feed(&cs);
        assert_eq!(p.recent.len(), MAX_RECENT);
        assert_eq!(p.total_fed, 100);
    }

    #[test]
    fn stage_transitions() {
        let mut p = Pet::new(0);
        assert_eq!(p.stage(), Stage::Egg);
        p.total_fed = BABY_AT - 1;
        assert_eq!(p.stage(), Stage::Egg);
        p.total_fed = BABY_AT;
        assert_eq!(p.stage(), Stage::Baby);
        p.total_fed = ADULT_AT - 1;
        assert_eq!(p.stage(), Stage::Baby);
        p.total_fed = ADULT_AT;
        assert_eq!(p.stage(), Stage::Adult);
    }

    #[test]
    fn starving_pet_is_sleepy_and_recovers_when_fed() {
        let mut p = Pet::new(0);
        p.hunger = 100.0;
        p.health = 10.0;
        assert_eq!(p.mood(), Mood::Sleepy);
        let cs: Vec<_> = (0..10).map(commit).collect();
        p.feed(&cs);
        assert_ne!(p.mood(), Mood::Sleepy);
    }

    #[test]
    fn clock_going_backwards_is_harmless() {
        let mut p = Pet::new(1000);
        let before = p.clone();
        p.tick(500);
        assert_eq!(p.hunger, before.hunger);
        assert_eq!(p.health, before.health);
        assert_eq!(p.last_update, 500);
        p.tick(500);
        assert_eq!(p.hunger, before.hunger);
    }

    #[test]
    fn extreme_timestamps_do_not_panic() {
        let mut p = Pet::new(i64::MAX);
        p.tick(i64::MIN);
        p.tick(i64::MAX);
        let mut q = Pet::new(i64::MIN);
        q.tick(i64::MAX);
        assert_eq!(q.hunger, 100.0);
    }

    #[test]
    fn sanitize_fixes_bad_values() {
        let mut p = Pet::new(0);
        p.hunger = f64::NAN;
        p.health = 500.0;
        p.sanitize();
        assert_eq!(p.hunger, 0.0);
        assert_eq!(p.health, 100.0);
    }
}
