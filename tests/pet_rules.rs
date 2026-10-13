//! Pet rules from the spec: hunger, decay, feeding, stages, clock edge cases,
//! and decay across app restarts (save, then load later).

use gitagochi::pet::*;
use gitagochi::store;

fn fc(n: u32) -> FedCommit {
    FedCommit {
        id: format!("id{n}"),
        summary: format!("c{n}"),
        time: 0,
    }
}

#[test]
fn hunger_rises_with_time() {
    let mut p = Pet::new(0);
    let before = p.hunger;
    p.tick(3600);
    assert!(p.hunger > before);
}

#[test]
fn hunger_is_capped_and_health_decays_when_hungry_too_long() {
    let mut p = Pet::new(0);
    p.tick(100 * 3600);
    assert!(p.hunger <= 100.0);
    assert!(p.health < 100.0);
    assert!(p.health >= 0.0);
}

#[test]
fn health_does_not_decay_while_not_hungry() {
    let mut p = Pet::new(0);
    p.hunger = 0.0;
    p.tick(60);
    assert_eq!(p.health, 100.0);
}

#[test]
fn feeding_lowers_hunger_and_counts() {
    let mut p = Pet::new(0);
    let h = p.hunger;
    p.feed(&[fc(1)]);
    assert!(p.hunger < h);
    assert_eq!(p.total_fed, 1);
    assert_eq!(p.recent.len(), 1);
}

#[test]
fn feeding_never_goes_out_of_range() {
    let mut p = Pet::new(0);
    let many: Vec<_> = (0..500).map(fc).collect();
    p.feed(&many);
    assert!(p.hunger >= 0.0);
    assert!(p.health <= 100.0);
    assert_eq!(p.total_fed, 500);
    assert!(p.recent.len() <= MAX_RECENT);
}

#[test]
fn stages_progress_egg_baby_adult() {
    let mut p = Pet::new(0);
    assert_eq!(p.stage(), Stage::Egg);
    p.total_fed = BABY_AT;
    assert_eq!(p.stage(), Stage::Baby);
    p.total_fed = ADULT_AT;
    assert_eq!(p.stage(), Stage::Adult);
}

#[test]
fn no_death_starving_pet_is_sleepy_and_recovers() {
    let mut p = Pet::new(0);
    p.tick(10_000 * 3600);
    assert_eq!(p.mood(), Mood::Sleepy);
    assert!(p.health >= 0.0);
    let many: Vec<_> = (0..40).map(fc).collect();
    p.feed(&many);
    assert_ne!(p.mood(), Mood::Sleepy);
}

#[test]
fn clock_going_backwards_does_not_panic_or_decay() {
    let mut p = Pet::new(10_000);
    let (h, hp) = (p.hunger, p.health);
    p.tick(5_000);
    assert_eq!(p.hunger, h);
    assert_eq!(p.health, hp);
    // time resumes from the new reference, not the old future one
    p.tick(5_000 + 3600);
    assert!(p.hunger > h);
}

#[test]
fn extreme_clock_values_do_not_panic() {
    let mut p = Pet::new(i64::MAX);
    p.tick(i64::MIN);
    p.tick(i64::MAX);
    let mut p = Pet::new(i64::MIN);
    p.tick(i64::MAX);
    assert!(p.hunger.is_finite() && p.health.is_finite());
}

#[test]
fn decay_continues_across_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("pet.json");
    let mut p = Pet::new(1_000);
    p.hunger = 0.0;
    store::save(&path, &p).unwrap();
    // app closed for 6 hours
    let mut loaded = store::load(&path, 1_000 + 6 * 3600);
    assert_eq!(loaded.last_update, 1_000);
    loaded.tick(1_000 + 6 * 3600);
    assert!(loaded.hunger > 50.0, "hunger was {}", loaded.hunger);
    // equals what an uninterrupted pet would have
    let mut cont = p.clone();
    cont.tick(1_000 + 6 * 3600);
    assert!((cont.hunger - loaded.hunger).abs() < 1e-9);
}

#[test]
fn restart_with_clock_set_back_is_safe() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("pet.json");
    let p = Pet::new(1_000_000);
    store::save(&path, &p).unwrap();
    let mut loaded = store::load(&path, 5);
    loaded.tick(5);
    assert!(loaded.hunger.is_finite());
}
