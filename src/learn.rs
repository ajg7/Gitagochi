//! Learn screen: navigation state, key handling and drawing.

use crate::academy::curriculum::{self, MODULES, Module, ObjId};
use crate::academy::grade::{CheckResult, GradeError, Report};
use crate::academy::progress::Progress;
use crate::academy::quiz::{self, Score};
use crate::git::strip_control;
use ratatui::Frame;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use std::collections::HashMap;
use std::path::PathBuf;

pub const MENU: [&str; 4] = [
    "Objectives and lesson",
    "Project brief",
    "Take the quiz",
    "Run the project check",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Doc {
    Lesson,
    Brief,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Nav {
    Pet,
    List,
    Menu(u8),
    Reader(u8, Doc),
    Quiz {
        m: u8,
        order: Vec<usize>,
        answers: Vec<usize>,
    },
    QuizResult {
        m: u8,
        order: Vec<usize>,
        score: Score,
    },
    Check(u8),
    Setup,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    None,
    Quit,
    Refresh,
    SaveProgress,
    StartCheck(u8),
    Scaffold(PathBuf),
}

/// Learn screen state.
pub struct Learn {
    pub nav: Nav,
    pub sel: usize,
    pub scroll: u16,
    pub note: String,
    pub input: String,
    pub running: Option<u8>,
    pub reports: HashMap<u8, Result<Report, String>>,
    last_order: HashMap<u8, Vec<usize>>,
}

fn module(m: u8) -> &'static Module {
    &MODULES[usize::from(m) - 1]
}

fn status(p: &Progress, m: u8) -> &'static str {
    if p.passed(m) {
        "passed"
    } else if p.open(m) {
        "open"
    } else {
        "locked"
    }
}

impl Learn {
    pub fn new(default_project: PathBuf) -> Learn {
        Learn {
            nav: Nav::Pet,
            sel: 0,
            scroll: 0,
            note: String::new(),
            input: default_project.to_string_lossy().into_owned(),
            running: None,
            reports: HashMap::new(),
            last_order: HashMap::new(),
        }
    }

    fn go(&mut self, nav: Nav) {
        self.nav = nav;
        self.scroll = 0;
        self.note.clear();
    }

    fn start_quiz(&mut self, m: u8, seed: u64) {
        let n = module(m).quiz.len();
        let order = quiz::order(n, self.last_order.get(&m).map(Vec::as_slice), seed);
        self.last_order.insert(m, order.clone());
        self.go(Nav::Quiz {
            m,
            order,
            answers: Vec::new(),
        });
    }

    fn start_check(&mut self, m: u8, p: &Progress) -> Action {
        if p.project.is_none() {
            self.note = "No project yet: press p to set one up".into();
            return Action::None;
        }
        self.go(Nav::Check(m));
        if self.running.is_some() {
            self.note = "A check is already running".into();
            return Action::None;
        }
        Action::StartCheck(m)
    }

    /// Show a grading result for module `m`.
    pub fn check_done(&mut self, m: u8, r: Result<Report, GradeError>) {
        self.running = None;
        self.reports.insert(m, r.map_err(|e| e.to_string()));
    }

    /// Scaffolding failed; stay on the setup screen with the reason.
    pub fn setup_failed(&mut self, msg: String) {
        self.note = msg;
    }

    /// Scaffolding worked; return to the module list.
    pub fn setup_done(&mut self, msg: String) {
        self.go(Nav::List);
        self.note = msg;
    }

    /// Handle one key press. Ctrl-C is handled by the caller.
    pub fn handle_key(&mut self, key: KeyCode, p: &mut Progress, seed: u64) -> Action {
        if self.nav == Nav::Setup {
            return self.setup_key(key);
        }
        match key {
            KeyCode::Char('q') => return Action::Quit,
            KeyCode::Char('p') if self.nav != Nav::Pet => {
                self.go(Nav::Setup);
                return Action::None;
            }
            KeyCode::Up => {
                self.sel = self.sel.saturating_sub(1);
                self.scroll = self.scroll.saturating_sub(1);
            }
            KeyCode::Down => {
                self.sel += 1;
                self.scroll = self.scroll.saturating_add(1);
            }
            KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(10),
            KeyCode::PageDown => self.scroll = self.scroll.saturating_add(10),
            _ => {}
        }
        match self.nav.clone() {
            Nav::Pet => match key {
                KeyCode::Esc => return Action::Quit,
                KeyCode::Char('r') => return Action::Refresh,
                KeyCode::Char('l') => {
                    self.go(Nav::List);
                    self.sel = 0;
                }
                _ => {}
            },
            Nav::List => {
                self.sel = self.sel.min(MODULES.len().saturating_sub(1));
                match key {
                    KeyCode::Esc => self.go(Nav::Pet),
                    KeyCode::Enter => {
                        let m = self.sel as u8 + 1;
                        if p.open(m) {
                            self.go(Nav::Menu(m));
                            self.sel = 0;
                        } else {
                            self.note =
                                format!("Module {m} is locked: pass module {} first", m - 1);
                        }
                    }
                    _ => {}
                }
            }
            Nav::Menu(m) => {
                self.sel = self.sel.min(MENU.len() - 1);
                match key {
                    KeyCode::Esc => {
                        self.go(Nav::List);
                        self.sel = usize::from(m) - 1;
                    }
                    KeyCode::Enter => match self.sel {
                        0 => self.go(Nav::Reader(m, Doc::Lesson)),
                        1 => self.go(Nav::Reader(m, Doc::Brief)),
                        2 => self.start_quiz(m, seed),
                        _ => return self.start_check(m, p),
                    },
                    _ => {}
                }
            }
            Nav::Reader(m, _) => {
                if key == KeyCode::Esc {
                    self.back_to_menu(m);
                }
            }
            Nav::Quiz {
                m,
                order,
                mut answers,
            } => {
                let choice = match key {
                    KeyCode::Char(c @ '1'..='9') => Some(c as usize - '1' as usize),
                    KeyCode::Char(c @ 'a'..='i') => Some(c as usize - 'a' as usize),
                    _ => None,
                };
                if key == KeyCode::Esc {
                    self.back_to_menu(m);
                    return Action::None;
                }
                let items = module(m).quiz;
                let Some(c) = choice else { return Action::None };
                if c >= items[order[answers.len()]].choices.len() {
                    return Action::None;
                }
                answers.push(c);
                if answers.len() < order.len() {
                    self.nav = Nav::Quiz { m, order, answers };
                    return Action::None;
                }
                let shown: Vec<_> = order.iter().map(|&i| items[i]).collect();
                let score = quiz::score(&shown, &answers);
                p.record_quiz(m, score.percent);
                self.go(Nav::QuizResult { m, order, score });
                return Action::SaveProgress;
            }
            Nav::QuizResult { m, .. } => match key {
                KeyCode::Esc => self.back_to_menu(m),
                KeyCode::Char('r') => self.start_quiz(m, seed),
                _ => {}
            },
            Nav::Check(m) => match key {
                KeyCode::Esc => self.back_to_menu(m),
                KeyCode::Char('r') => return self.start_check(m, p),
                _ => {}
            },
            Nav::Setup => {}
        }
        Action::None
    }

    fn back_to_menu(&mut self, m: u8) {
        self.go(Nav::Menu(m));
        self.sel = 0;
    }

    fn setup_key(&mut self, key: KeyCode) -> Action {
        match key {
            KeyCode::Esc => self.go(Nav::List),
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Char(c) => self.input.push(c),
            KeyCode::Enter => {
                let path = self.input.trim();
                if path.is_empty() {
                    self.note = "Type a folder path first".into();
                } else {
                    return Action::Scaffold(PathBuf::from(path));
                }
            }
            _ => {}
        }
        Action::None
    }
}

fn objective_line(id: ObjId) -> String {
    match curriculum::objective(id) {
        Some(o) => format!(
            "objective {id} {}; revisit: module {}, {}",
            o.text, id.0, o.section
        ),
        None => format!("objective {id}"),
    }
}

/// Lines describing a submitted quiz: the score and each missed item.
pub fn quiz_lines(m: u8, order: &[usize], score: &Score) -> Vec<String> {
    let verdict = if score.passed {
        "passed"
    } else {
        "not passed (80% needed)"
    };
    let mut out = vec![format!(
        "Score: {}/{} ({}%) - {verdict}",
        score.correct, score.total, score.percent
    )];
    let items = module(m).quiz;
    for &i in &score.missed {
        let item = &items[order[i]];
        out.push(String::new());
        out.push(format!(
            "Missed: {}",
            item.prompt.lines().next().unwrap_or("")
        ));
        for t in item.tags {
            out.push(format!("  {}", objective_line(*t)));
        }
    }
    out
}

/// Lines describing one check result.
pub fn check_lines(r: &CheckResult) -> Vec<String> {
    if r.passed {
        return vec![format!("ok    {} (module {})", r.name, r.module)];
    }
    let mut out = vec![format!("FAIL  {} (module {})", r.name, r.module)];
    out.extend(
        r.message
            .lines()
            .map(|l| format!("      {}", strip_control(l))),
    );
    for t in &r.tags {
        out.push(format!("      {}", objective_line(*t)));
    }
    out
}

fn report_lines(r: &Result<Report, String>) -> Vec<String> {
    match r {
        Err(e) => vec![format!("Cannot check the project: {e}")],
        Ok(rep) => {
            let short: String = rep.commit.chars().take(7).collect();
            let failing = rep.results.iter().filter(|c| !c.passed).count();
            let mut out = vec![if rep.passed() {
                format!("Commit {short}: all {} checks passed", rep.results.len())
            } else {
                format!(
                    "Commit {short}: {failing} of {} checks failed",
                    rep.results.len()
                )
            }];
            out.push(String::new());
            for c in &rep.results {
                out.extend(check_lines(c));
            }
            out
        }
    }
}

fn lesson_text(m: &Module) -> Vec<String> {
    let mut out = vec!["Objectives:".to_string()];
    for o in m.objectives {
        out.push(format!("  {} {} (section: {})", o.id, o.text, o.section));
    }
    for s in m.sections {
        out.push(String::new());
        out.push(format!("== {} ==", s.title));
        out.extend(s.body.lines().map(str::to_string));
    }
    if !m.read_more.is_empty() {
        out.push(String::new());
        out.push("Read more (optional):".into());
        out.extend(m.read_more.iter().map(|r| format!("  {r}")));
    }
    out
}

fn body(l: &Learn, p: &Progress) -> (String, Vec<String>, &'static str) {
    match &l.nav {
        Nav::Pet | Nav::List => {
            let sel = l.sel.min(MODULES.len().saturating_sub(1));
            let lines = MODULES
                .iter()
                .enumerate()
                .map(|(i, m)| {
                    let n = i as u8 + 1;
                    let cur = if i == sel { ">" } else { " " };
                    format!("{cur} [{:<6}] {n:>2}. {}", status(p, n), m.title)
                })
                .collect();
            (
                " Rust academy ".into(),
                lines,
                "Up/Down select  Enter open  p project setup  Esc back  q quit",
            )
        }
        Nav::Menu(m) => {
            let md = module(*m);
            let mp = p.get(*m);
            let mut lines = vec![
                format!("Status: {}", status(p, *m)),
                format!(
                    "Quiz best: {}%   Project part: {}",
                    mp.quiz_best,
                    if mp.project_passed {
                        "passed"
                    } else {
                        "not passed"
                    }
                ),
                format!(
                    "Project: {}",
                    p.project
                        .as_ref()
                        .map_or("not set up (press p)".into(), |x| x.display().to_string())
                ),
                String::new(),
            ];
            let sel = l.sel.min(MENU.len() - 1);
            for (i, item) in MENU.iter().enumerate() {
                lines.push(format!("{} {item}", if i == sel { ">" } else { " " }));
            }
            (
                format!(" Module {m}: {} ", md.title),
                lines,
                "Up/Down select  Enter open  p project setup  Esc back  q quit",
            )
        }
        Nav::Reader(m, doc) => {
            let md = module(*m);
            let (title, lines) = match doc {
                Doc::Lesson => ("Lesson", lesson_text(md)),
                Doc::Brief => (
                    "Project brief",
                    md.brief.lines().map(str::to_string).collect(),
                ),
            };
            (
                format!(" Module {m}: {title} "),
                lines,
                "Up/Down PgUp/PgDn scroll  Esc back  q quit",
            )
        }
        Nav::Quiz { m, order, answers } => {
            let item = &module(*m).quiz[order[answers.len()]];
            let mut lines = vec![
                format!("Question {} of {}", answers.len() + 1, order.len()),
                String::new(),
            ];
            lines.extend(item.prompt.lines().map(str::to_string));
            lines.push(String::new());
            for (i, c) in item.choices.iter().enumerate() {
                lines.push(format!("  {}) {c}", i + 1));
            }
            (
                format!(" Module {m}: quiz "),
                lines,
                "1-4 or a-d answer  Esc abandon  q quit",
            )
        }
        Nav::QuizResult { m, order, score } => (
            format!(" Module {m}: quiz result "),
            quiz_lines(*m, order, score),
            "r retake  Up/Down scroll  Esc back  q quit",
        ),
        Nav::Check(m) => {
            let lines = if l.running == Some(*m) {
                vec!["Checking your last commit... (the pet keeps living meanwhile)".into()]
            } else if let Some(r) = l.reports.get(m) {
                report_lines(r)
            } else {
                vec!["No check run yet; press r to run it".into()]
            };
            (
                format!(" Module {m}: project check "),
                lines,
                "r rerun  Up/Down scroll  Esc back  q quit",
            )
        }
        Nav::Setup => {
            let mut lines = vec![
                "Type the folder for your tracker project (new or empty), then press Enter."
                    .to_string(),
                String::new(),
                format!("> {}_", l.input),
            ];
            if let Some(cur) = &p.project {
                lines.push(String::new());
                lines.push(format!("Current project: {}", cur.display()));
            }
            (
                " Project setup ".into(),
                lines,
                "Enter create  Backspace delete  Esc cancel",
            )
        }
    }
}

/// Draw the learn screen; `pet_status` is a one-line live pet summary.
pub fn draw(f: &mut Frame, l: &Learn, p: &Progress, pet_status: &str) {
    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(3),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(f.area());
    f.render_widget(Paragraph::new(pet_status.to_string()), rows[0]);
    let (title, lines, hint) = body(l, p);
    let lines: Vec<Line> = lines.into_iter().map(Line::from).collect();
    let scroll = match l.nav {
        Nav::Reader(..) | Nav::QuizResult { .. } | Nav::Check(_) => l.scroll,
        _ => 0,
    };
    f.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(title))
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0)),
        rows[1],
    );
    f.render_widget(Paragraph::new(l.note.clone()), rows[2]);
    f.render_widget(Paragraph::new(hint), rows[3]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::academy::grade::CheckResult;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn learn() -> Learn {
        Learn::new(PathBuf::from("tracker"))
    }

    fn press(l: &mut Learn, p: &mut Progress, keys: &[KeyCode]) -> Action {
        let mut last = Action::None;
        for (i, k) in keys.iter().enumerate() {
            last = l.handle_key(*k, p, i as u64);
        }
        last
    }

    fn passed_m1() -> Progress {
        let mut p = Progress::default();
        p.record_quiz(1, 100);
        p.record_project_pass(1);
        p
    }

    fn screen(l: &Learn, p: &Progress) -> String {
        let mut t = Terminal::new(TestBackend::new(100, 40)).unwrap();
        t.draw(|f| draw(f, l, p, "pet")).unwrap();
        let buf = t.backend().buffer();
        (0..buf.area.height)
            .map(|y| {
                (0..buf.area.width)
                    .map(|x| buf[(x, y)].symbol().to_string())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn lrn_09_a_l_opens_list_esc_returns() {
        let (mut l, mut p) = (learn(), Progress::default());
        press(&mut l, &mut p, &[KeyCode::Char('l')]);
        assert_eq!(l.nav, Nav::List);
        press(&mut l, &mut p, &[KeyCode::Esc]);
        assert_eq!(l.nav, Nav::Pet);
    }

    #[test]
    fn lrn_10_a_lesson_brief_quiz_check_reachable() {
        let (mut l, mut p) = (learn(), Progress::default());
        press(&mut l, &mut p, &[KeyCode::Char('l'), KeyCode::Enter]);
        assert_eq!(l.nav, Nav::Menu(1));
        press(&mut l, &mut p, &[KeyCode::Enter]);
        assert_eq!(l.nav, Nav::Reader(1, Doc::Lesson));
        assert!(screen(&l, &p).contains("Cargo projects"));
        press(
            &mut l,
            &mut p,
            &[KeyCode::Esc, KeyCode::Down, KeyCode::Enter],
        );
        assert_eq!(l.nav, Nav::Reader(1, Doc::Brief));
        press(
            &mut l,
            &mut p,
            &[KeyCode::Esc, KeyCode::Down, KeyCode::Down, KeyCode::Enter],
        );
        assert!(matches!(l.nav, Nav::Quiz { m: 1, .. }));
        press(&mut l, &mut p, &[KeyCode::Esc]);
        for _ in 0..3 {
            press(&mut l, &mut p, &[KeyCode::Down]);
        }
        assert_eq!(press(&mut l, &mut p, &[KeyCode::Enter]), Action::None);
        assert!(l.note.contains("press p"));
        p.project = Some(PathBuf::from("proj"));
        assert_eq!(
            press(&mut l, &mut p, &[KeyCode::Enter]),
            Action::StartCheck(1)
        );
        assert_eq!(l.nav, Nav::Check(1));
        press(&mut l, &mut p, &[KeyCode::Char('p')]);
        assert_eq!(l.nav, Nav::Setup);
        let a = press(&mut l, &mut p, &[KeyCode::Char('q'), KeyCode::Enter]);
        assert_eq!(a, Action::Scaffold(PathBuf::from("trackerq")));
    }

    #[test]
    fn lrn_11_a_module_2_locked() {
        let (mut l, mut p) = (learn(), Progress::default());
        press(
            &mut l,
            &mut p,
            &[KeyCode::Char('l'), KeyCode::Down, KeyCode::Enter],
        );
        assert_eq!(l.nav, Nav::List);
        assert!(l.note.contains("locked"));
        let mut q = passed_m1();
        press(&mut l, &mut q, &[KeyCode::Enter]);
        assert_eq!(l.nav, Nav::Menu(2));
    }

    fn take_quiz(l: &mut Learn, p: &mut Progress, wrong: usize) -> Action {
        let Nav::Quiz { m, order, .. } = l.nav.clone() else {
            panic!("not in a quiz")
        };
        let items = module(m).quiz;
        let mut last = Action::None;
        for (k, &i) in order.iter().enumerate() {
            let ans = if k < wrong {
                (items[i].answer + 1) % items[i].choices.len()
            } else {
                items[i].answer
            };
            last = l.handle_key(KeyCode::Char((b'1' + ans as u8) as char), p, 0);
        }
        last
    }

    #[test]
    fn lrn_19_a_result_names_missed_objective_and_section() {
        let (mut l, mut p) = (learn(), Progress::default());
        press(
            &mut l,
            &mut p,
            &[
                KeyCode::Char('l'),
                KeyCode::Enter,
                KeyCode::Down,
                KeyCode::Down,
            ],
        );
        press(&mut l, &mut p, &[KeyCode::Enter]);
        assert_eq!(take_quiz(&mut l, &mut p, 1), Action::SaveProgress);
        let Nav::QuizResult { m, order, score } = l.nav.clone() else {
            panic!("no result")
        };
        assert_eq!(score.missed.len(), 1);
        let lines = quiz_lines(m, &order, &score).join("\n");
        assert!(lines.contains("Score: 9/10 (90%)"), "{lines}");
        let item = &MODULES[0].quiz[order[score.missed[0]]];
        let obj = curriculum::objective(item.tags[0]).unwrap();
        assert!(
            lines.contains(obj.text) && lines.contains(obj.section),
            "{lines}"
        );
        assert_eq!(p.get(1).quiz_best, 90);
    }

    #[test]
    fn lrn_20_a_retake_after_fail() {
        let (mut l, mut p) = (learn(), Progress::default());
        press(
            &mut l,
            &mut p,
            &[
                KeyCode::Char('l'),
                KeyCode::Enter,
                KeyCode::Down,
                KeyCode::Down,
            ],
        );
        press(&mut l, &mut p, &[KeyCode::Enter]);
        let Nav::Quiz { order: first, .. } = l.nav.clone() else {
            panic!()
        };
        take_quiz(&mut l, &mut p, 5);
        assert!(matches!(&l.nav, Nav::QuizResult { score, .. } if !score.passed));
        press(&mut l, &mut p, &[KeyCode::Char('r')]);
        let Nav::Quiz {
            order: second,
            answers,
            ..
        } = l.nav.clone()
        else {
            panic!("retake did not start")
        };
        assert!(answers.is_empty());
        assert_ne!(first, second);
    }

    #[test]
    fn lrn_17_a_check_lines_show_name_message_section() {
        let r = CheckResult {
            module: 1,
            name: "m01_counts_non_empty_lines".into(),
            passed: false,
            message: "count_lines should count only the non-blank lines".into(),
            tags: vec![ObjId(1, 1)],
        };
        let s = check_lines(&r).join("\n");
        assert!(s.contains("m01_counts_non_empty_lines"));
        assert!(s.contains("count_lines should count only the non-blank lines"));
        assert!(s.contains("revisit: module 1, Cargo projects"), "{s}");
    }

    #[test]
    fn lrn_01_a_lists_all_modules_in_order() {
        let (mut l, mut p) = (learn(), passed_m1());
        press(&mut l, &mut p, &[KeyCode::Char('l')]);
        let s = screen(&l, &p);
        let mut at = 0;
        for (i, m) in MODULES.iter().enumerate() {
            let pos = s[at..]
                .find(m.title)
                .unwrap_or_else(|| panic!("{}", m.title))
                + at;
            let line = s[..pos].rsplit('\n').next().unwrap();
            let want = match i {
                0 => "passed",
                1 => "open",
                _ => "locked",
            };
            assert!(line.contains(want), "{line}");
            at = pos;
        }
    }

    #[test]
    fn lrn_22_a_passed_module_reopens() {
        let (mut l, mut p) = (learn(), passed_m1());
        press(&mut l, &mut p, &[KeyCode::Char('l'), KeyCode::Enter]);
        assert_eq!(l.nav, Nav::Menu(1));
        press(
            &mut l,
            &mut p,
            &[KeyCode::Down, KeyCode::Down, KeyCode::Enter],
        );
        take_quiz(&mut l, &mut p, 10);
        assert!(p.passed(1));
        assert_eq!(p.get(1).quiz_best, 100);
    }
}
