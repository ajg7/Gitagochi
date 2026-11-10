//! Spec 003 acceptance tests driven through the public API (learn screen, quiz, badges, content).

use gitagochi::academy::curriculum::{CheckKind, MODULES};
use gitagochi::academy::progress::Progress;
use gitagochi::learn::{Action, Learn, Nav, quiz_lines};
use gitagochi::pet::Pet;
use gitagochi::ui::{View, draw};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use std::path::PathBuf;

fn learn() -> Learn {
    Learn::new(PathBuf::from("tracker"))
}

fn press(l: &mut Learn, p: &mut Progress, keys: &[KeyCode], seed: u64) -> Action {
    let mut last = Action::None;
    for k in keys {
        last = l.handle_key(*k, p, seed);
    }
    last
}

fn passed(n: u8) -> Progress {
    let mut p = Progress::default();
    for m in 1..=n {
        p.record_quiz(m, 100);
        p.record_project_pass(m);
    }
    p
}

/// Answer the quiz currently shown: `pick(item_index_in_module, displayed_choice_positions)`.
fn answer_quiz(
    l: &mut Learn,
    p: &mut Progress,
    seed: u64,
    pick: impl Fn(usize, &[usize]) -> usize,
) {
    let Nav::Quiz {
        m, order, choices, ..
    } = l.nav.clone()
    else {
        panic!("not in a quiz: {:?}", l.nav);
    };
    let items = MODULES[usize::from(m) - 1].quiz;
    for (k, &i) in order.iter().enumerate() {
        let pos = pick(i, &choices[k]);
        let key = KeyCode::Char((b'1' + pos as u8) as char);
        l.handle_key(key, p, seed);
    }
    assert!(
        matches!(l.nav, Nav::QuizResult { .. }),
        "{} items",
        items.len()
    );
}

fn open_quiz(l: &mut Learn, p: &mut Progress, m: u8, seed: u64) {
    l.nav = Nav::Pet;
    let mut keys = vec![KeyCode::Char('l')];
    keys.extend(std::iter::repeat_n(KeyCode::Down, usize::from(m) - 1));
    keys.extend([KeyCode::Enter, KeyCode::Down, KeyCode::Down, KeyCode::Enter]);
    press(l, p, &keys, seed);
    assert!(matches!(l.nav, Nav::Quiz { .. }), "{:?}", l.nav);
}

#[test]
fn lrn_01_a_all_modules_listed_in_order_offline() {
    // The curriculum is compiled in; the list screen shows every title in order.
    let mut l = learn();
    let mut p = Progress::default();
    press(&mut l, &mut p, &[KeyCode::Char('l')], 0);
    let mut t = Terminal::new(TestBackend::new(120, 40)).unwrap();
    t.draw(|f| gitagochi::learn::draw(f, &l, &p, "pet"))
        .unwrap();
    let buf = t.backend().buffer();
    let text: String = (0..buf.area.height)
        .map(|y| {
            (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol().to_string())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut last = 0;
    for m in MODULES {
        let at = text
            .find(m.title)
            .unwrap_or_else(|| panic!("{} not listed", m.title));
        assert!(at >= last, "{} out of order", m.title);
        last = at;
    }
}

#[test]
fn lrn_09_a_l_opens_list_and_esc_returns() {
    let (mut l, mut p) = (learn(), Progress::default());
    press(&mut l, &mut p, &[KeyCode::Char('l')], 0);
    assert_eq!(l.nav, Nav::List);
    press(&mut l, &mut p, &[KeyCode::Esc], 0);
    assert_eq!(l.nav, Nav::Pet);
}

#[test]
fn lrn_10_a_all_four_views_reachable() {
    let (mut l, mut p) = (learn(), Progress::default());
    p.project = Some(PathBuf::from("proj"));
    press(&mut l, &mut p, &[KeyCode::Char('l'), KeyCode::Enter], 0);
    assert!(matches!(l.nav, Nav::Menu(1)));
    let mut seen = 0;
    for steps in 0..4 {
        l.nav = Nav::Menu(1);
        l.sel = 0;
        let mut keys = vec![KeyCode::Down; steps];
        keys.push(KeyCode::Enter);
        let a = press(&mut l, &mut p, &keys, 0);
        let ok = match steps {
            0 | 1 => matches!(l.nav, Nav::Reader(1, _)),
            2 => matches!(l.nav, Nav::Quiz { m: 1, .. }),
            _ => a == Action::StartCheck(1) && l.nav == Nav::Check(1),
        };
        assert!(ok, "menu entry {steps}: {:?}", l.nav);
        seen += 1;
    }
    assert_eq!(seen, 4);
}

#[test]
fn lrn_11_a_module_2_locked_until_module_1_passed() {
    let (mut l, mut p) = (learn(), Progress::default());
    press(
        &mut l,
        &mut p,
        &[KeyCode::Char('l'), KeyCode::Down, KeyCode::Enter],
        0,
    );
    assert_eq!(l.nav, Nav::List, "locked module must not open");
    assert!(l.note.contains("locked"), "{}", l.note);
    // Quiz alone does not unlock it.
    p.record_quiz(1, 100);
    press(&mut l, &mut p, &[KeyCode::Enter], 0);
    assert_eq!(l.nav, Nav::List);
    p.record_project_pass(1);
    press(&mut l, &mut p, &[KeyCode::Enter], 0);
    assert_eq!(l.nav, Nav::Menu(2));
}

#[test]
fn lrn_14_a_both_parts_required() {
    let mut p = Progress::default();
    p.record_quiz(2, 100);
    assert!(!p.passed(2));
    let mut q = Progress::default();
    q.record_project_pass(2);
    assert!(!q.passed(2));
    q.record_quiz(2, 80);
    assert!(q.passed(2));
}

#[test]
fn lrn_18_a_pass_mark_on_real_quizzes() {
    for m in MODULES {
        let n = m.quiz.len();
        let need = (n * 4).div_ceil(5);
        let ans = |wrong: usize| -> Vec<usize> {
            (0..n)
                .map(|i| {
                    if i < wrong {
                        m.quiz[i].answer + 1
                    } else {
                        m.quiz[i].answer
                    }
                })
                .collect()
        };
        let ok = gitagochi::academy::quiz::score(m.quiz, &ans(n - need));
        assert!(ok.passed, "{}: {} of {n} should pass", m.title, need);
        let bad = gitagochi::academy::quiz::score(m.quiz, &ans(n - need + 1));
        assert!(!bad.passed, "{}: {} of {n} should fail", m.title, need - 1);
    }
}

#[test]
fn lrn_19_a_missed_item_names_objective_and_section() {
    let (mut l, mut p) = (learn(), Progress::default());
    open_quiz(&mut l, &mut p, 1, 5);
    let miss = std::cell::Cell::new(true);
    answer_quiz(&mut l, &mut p, 5, |i, ch| {
        let right = ch
            .iter()
            .position(|&c| c == MODULES[0].quiz[i].answer)
            .unwrap();
        if miss.replace(false) {
            (right + 1) % ch.len()
        } else {
            right
        }
    });
    let Nav::QuizResult { m, order, score } = l.nav.clone() else {
        unreachable!()
    };
    assert_eq!(score.missed.len(), 1);
    let text = quiz_lines(m, &order, &score).join("\n");
    assert!(
        text.contains(&format!("{}/{}", score.correct, score.total)),
        "{text}"
    );
    let item = &MODULES[0].quiz[order[score.missed[0]]];
    let o = gitagochi::academy::curriculum::objective(item.tags[0]).unwrap();
    assert!(text.contains(&item.tags[0].to_string()), "{text}");
    assert!(text.contains(o.section), "{text}");
}

#[test]
fn lrn_20_a_retake_after_fail_reorders_items() {
    let (mut l, mut p) = (learn(), Progress::default());
    open_quiz(&mut l, &mut p, 1, 11);
    let Nav::Quiz { order: first, .. } = l.nav.clone() else {
        unreachable!()
    };
    // Fail on purpose.
    answer_quiz(&mut l, &mut p, 11, |i, ch| {
        let right = ch
            .iter()
            .position(|&c| c == MODULES[0].quiz[i].answer)
            .unwrap();
        (right + 1) % ch.len()
    });
    let Nav::QuizResult { score, .. } = l.nav.clone() else {
        unreachable!()
    };
    assert!(!score.passed);
    for seed in [11, 12, 13] {
        press(&mut l, &mut p, &[KeyCode::Char('r')], seed);
        let Nav::Quiz { order, .. } = l.nav.clone() else {
            panic!("retake did not start")
        };
        assert_ne!(order, first, "seed {seed}");
        answer_quiz(&mut l, &mut p, seed, |_, _| 0);
    }
}

#[test]
fn lrn_20_b_fixed_position_strategies_never_certain() {
    for (mi, m) in MODULES.iter().enumerate() {
        let max = m.quiz.iter().map(|q| q.choices.len()).max().unwrap();
        for pos in 0..max {
            let mut fails = 0;
            let mut correct_positions = std::collections::HashSet::new();
            for seed in 0..60u64 {
                let (mut l, mut p) = (learn(), passed(mi as u8));
                open_quiz(&mut l, &mut p, mi as u8 + 1, seed.wrapping_mul(7919) + 1);
                let Nav::Quiz { order, choices, .. } = l.nav.clone() else {
                    unreachable!()
                };
                for (k, &i) in order.iter().enumerate() {
                    let r = choices[k]
                        .iter()
                        .position(|&c| c == m.quiz[i].answer)
                        .unwrap();
                    correct_positions.insert(r);
                }
                // Items with fewer choices get their last position instead.
                for c in &choices {
                    let key = (b'1' + pos.min(c.len() - 1) as u8) as char;
                    l.handle_key(KeyCode::Char(key), &mut p, 0);
                }
                if p.get(mi as u8 + 1).quiz_best < 80 {
                    fails += 1;
                }
            }
            assert!(fails > 0, "module {} position {pos} always passes", mi + 1);
            assert!(
                correct_positions.len() > 1,
                "module {}: correct position never varies",
                mi + 1
            );
        }
    }
}

#[test]
fn lrn_21_a_badge_visible_and_pet_unchanged() {
    let pet = Pet::new(1000);
    let before = pet.clone();
    let p = passed(1);
    let badges = p.badges();
    let mut t = Terminal::new(TestBackend::new(80, 24)).unwrap();
    t.draw(|f| {
        draw(
            f,
            &View {
                pet: &pet,
                repo_count: 1,
                author: "a@b.c",
                message: "",
                badges: &badges,
            },
        )
    })
    .unwrap();
    let buf = t.backend().buffer();
    let text: String = (0..buf.area.height)
        .flat_map(|y| (0..buf.area.width).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();
    assert!(text.contains("Badges") && text.contains("[1]"), "{text}");
    assert_eq!(pet, before);
}

#[test]
fn lrn_22_a_failed_retake_keeps_pass() {
    let (mut l, mut p) = (learn(), passed(1));
    open_quiz(&mut l, &mut p, 1, 3);
    answer_quiz(&mut l, &mut p, 3, |i, ch| {
        let right = ch
            .iter()
            .position(|&c| c == MODULES[0].quiz[i].answer)
            .unwrap();
        (right + 1) % ch.len()
    });
    assert!(p.passed(1));
    assert!(p.open(2));
    assert_eq!(p.badges(), vec![1]);
}

#[test]
fn lrn_28_hidden_tests_are_instrumentable() {
    // The grader rewrites exactly `fn name() {` once per test; any other shape fails closed.
    for m in MODULES {
        for c in m.checks {
            if let CheckKind::Test(n) = c.kind {
                let head = format!("fn {n}() {{");
                assert_eq!(m.hidden_tests.matches(&head).count(), 1, "{n}: {head:?}");
            }
        }
    }
}

#[test]
fn lrn_29_a_no_paths_default_is_current_dir_and_plain_dir_has_no_repo() {
    // Public pieces behind LRN-29: with no paths and no config the cwd is used; a plain
    // folder resolves to no repos (the app then opens with a notice instead of exiting).
    let cfg = gitagochi::repos::Config::default();
    assert_eq!(
        gitagochi::effective_paths(vec![], &cfg),
        vec![PathBuf::from(".")]
    );
    let d = tempfile::tempdir().unwrap();
    let r = gitagochi::repos::resolve(&[d.path().to_path_buf()]);
    assert!(r.repos.is_empty());
}
