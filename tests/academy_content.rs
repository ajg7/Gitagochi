//! Curriculum alignment rules over the shipped modules (LRN-02, LRN-05..08, LRN-17, LRN-18).

use gitagochi::academy::curriculum::{CheckKind, ItemKind, MODULES, ObjId};
use std::collections::HashSet;

fn taught(id: ObjId) -> bool {
    let Some(m) = MODULES.get(usize::from(id.0).wrapping_sub(1)) else {
        return false;
    };
    m.objectives.iter().any(|o| {
        o.id == id
            && m.sections
                .iter()
                .any(|s| s.title == o.section && !s.body.trim().is_empty())
    })
}

#[test]
fn lrn_02_a_every_module_complete() {
    for (i, m) in MODULES.iter().enumerate() {
        assert!(!m.title.is_empty(), "module {}", i + 1);
        assert!(!m.objectives.is_empty(), "{}: objectives", m.title);
        assert!(!m.sections.is_empty(), "{}: sections", m.title);
        assert!(!m.brief.trim().is_empty(), "{}: brief", m.title);
        assert!(!m.quiz.is_empty(), "{}: quiz", m.title);
        assert!(!m.checks.is_empty(), "{}: checks", m.title);
    }
}

#[test]
fn lrn_05_a_objectives_name_nonempty_sections() {
    for m in MODULES {
        for o in m.objectives {
            assert!(!o.text.trim().is_empty(), "{}: objective {}", m.title, o.id);
            let s = m.sections.iter().find(|s| s.title == o.section);
            let s = s.unwrap_or_else(|| panic!("{}: section {:?} missing", o.id, o.section));
            assert!(!s.body.trim().is_empty(), "{}: empty section", o.id);
        }
    }
}

#[test]
fn lrn_06_a_items_taught_in_lessons() {
    for m in MODULES {
        for it in m.quiz {
            for t in it.tags {
                assert!(taught(*t), "{}: {:?} tag {t} untaught", m.title, it.prompt);
            }
        }
        for c in m.checks {
            for t in c.tags {
                assert!(taught(*t), "{}: check tag {t} untaught", m.title);
            }
        }
        for r in m.read_more {
            assert!(r.starts_with("https://doc.rust-lang.org/book/"), "{r}");
        }
    }
}

#[test]
fn lrn_07_a_tags_known_and_not_later() {
    for (i, m) in MODULES.iter().enumerate() {
        let n = (i + 1) as u8;
        for (j, o) in m.objectives.iter().enumerate() {
            assert_eq!(o.id, ObjId(n, j as u8 + 1), "{}", m.title);
        }
        let tags = m
            .quiz
            .iter()
            .map(|q| q.tags)
            .chain(m.checks.iter().map(|c| c.tags));
        for ts in tags {
            assert!(!ts.is_empty(), "{}: untagged item", m.title);
            for t in ts {
                assert!(t.0 <= n, "{}: later tag {t}", m.title);
                assert!(taught(*t), "{}: unknown tag {t}", m.title);
            }
        }
    }
}

#[test]
fn lrn_08_a_every_objective_assessed() {
    for m in MODULES {
        for o in m.objectives {
            let hit = m.quiz.iter().any(|q| q.tags.contains(&o.id))
                || m.checks.iter().any(|c| c.tags.contains(&o.id));
            assert!(hit, "{}: objective {} unassessed", m.title, o.id);
        }
    }
}

#[test]
fn lrn_18_each_quiz_has_both_kinds() {
    for m in MODULES {
        assert!(
            m.quiz.iter().any(|q| q.kind == ItemKind::MultipleChoice),
            "{}",
            m.title
        );
        assert!(
            m.quiz.iter().any(|q| q.kind == ItemKind::SpotTheBug),
            "{}",
            m.title
        );
        for q in m.quiz {
            assert!(q.choices.len() >= 2, "{}: {}", m.title, q.prompt);
            assert!(q.answer < q.choices.len(), "{}: {}", m.title, q.prompt);
        }
    }
}

#[test]
fn lrn_17_hidden_tests_match_checks() {
    let mut all = HashSet::new();
    for (i, m) in MODULES.iter().enumerate() {
        let prefix = format!("m{:02}_", i + 1);
        let names: Vec<&str> = m
            .checks
            .iter()
            .filter_map(|c| match c.kind {
                CheckKind::Test(n) => Some(n),
                _ => None,
            })
            .collect();
        for n in &names {
            assert!(n.starts_with(&prefix), "{n}");
            assert!(all.insert(*n), "duplicate {n}");
            assert!(m.hidden_tests.contains(&format!("fn {n}(")), "{n} missing");
        }
        let defined = m
            .hidden_tests
            .match_indices("fn ")
            .map(|(p, _)| &m.hidden_tests[p + 3..])
            .filter_map(|s| s.split('(').next())
            .filter(|s| s.starts_with(&prefix));
        for d in defined {
            assert!(names.contains(&d), "hidden test {d} has no check");
        }
        assert!(!m.hidden_tests.contains("assert"), "{}: assert", m.title);
    }
}
