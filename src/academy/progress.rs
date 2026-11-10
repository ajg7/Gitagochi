//! Curriculum progress rules. Pure: no I/O.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Quiz percentage needed to pass (LRN-18).
pub const PASS_PERCENT: u8 = 80;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ModuleProgress {
    pub quiz_best: u8,
    pub project_passed: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Progress {
    pub modules: BTreeMap<u8, ModuleProgress>,
    pub project: Option<PathBuf>,
}

impl Progress {
    pub fn get(&self, m: u8) -> ModuleProgress {
        self.modules.get(&m).copied().unwrap_or_default()
    }

    pub fn passed(&self, m: u8) -> bool {
        let p = self.get(m);
        p.quiz_best >= PASS_PERCENT && p.project_passed
    }

    pub fn open(&self, m: u8) -> bool {
        (1..m).all(|k| self.passed(k))
    }

    pub fn record_quiz(&mut self, m: u8, percent: u8) {
        let e = self.modules.entry(m).or_default();
        e.quiz_best = e.quiz_best.max(percent.min(100));
    }

    pub fn record_project_pass(&mut self, m: u8) {
        self.modules.entry(m).or_default().project_passed = true;
    }

    pub fn badges(&self) -> Vec<u8> {
        self.modules
            .keys()
            .copied()
            .filter(|m| self.passed(*m))
            .collect()
    }

    /// Drop unknown modules (outside `1..=modules`), clamp scores, clear an empty path.
    pub fn sanitize(&mut self, modules: usize) {
        self.modules
            .retain(|k, _| *k >= 1 && usize::from(*k) <= modules);
        for p in self.modules.values_mut() {
            p.quiz_best = p.quiz_best.min(100);
        }
        if self
            .project
            .as_ref()
            .is_some_and(|p| p.as_os_str().is_empty())
        {
            self.project = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lrn_11_open_rule() {
        let mut p = Progress::default();
        assert!(p.open(1));
        assert!(!p.open(2));
        p.record_quiz(1, 80);
        assert!(!p.open(2));
        p.record_project_pass(1);
        assert!(p.open(2));
        assert!(!p.open(3));
    }

    #[test]
    fn lrn_14_a_both_parts_needed() {
        let mut p = Progress::default();
        p.record_quiz(1, 90);
        assert!(!p.passed(1));
        p.record_project_pass(1);
        assert!(p.passed(1));
        let mut q = Progress::default();
        q.record_project_pass(1);
        q.record_quiz(1, 70);
        assert!(!q.passed(1));
    }

    #[test]
    fn lrn_22_a_failed_retake_keeps_pass() {
        let mut p = Progress::default();
        p.record_quiz(1, 100);
        p.record_project_pass(1);
        p.record_quiz(1, 10);
        assert!(p.passed(1));
        assert_eq!(p.get(1).quiz_best, 100);
        assert_eq!(p.badges(), vec![1]);
    }

    #[test]
    fn sanitize_drops_unknown_and_clamps() {
        let mut p: Progress = serde_json::from_str(
            r#"{"modules":{"0":{"quiz_best":90},"1":{"quiz_best":250,"project_passed":true},"11":{}},"project":""}"#,
        )
        .unwrap();
        p.sanitize(10);
        assert_eq!(p.modules.keys().copied().collect::<Vec<_>>(), vec![1]);
        assert_eq!(p.get(1).quiz_best, 100);
        assert_eq!(p.project, None);
    }
}
