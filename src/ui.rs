//! Terminal UI: renders from state and runs the event loop.

use crate::academy::grade::{self, GradeEnv, GradeError, Job};
use crate::academy::progress::Progress;
use crate::academy::{quiz, scaffold};
use crate::git::Repo;
use crate::learn::{self, Action, Learn, Nav};
use crate::pet::{Mood, Pet, Stage};
use crate::repos::{self, Resolved};
use crate::{now_unix, store, sync_all};
use ratatui::Frame;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, Paragraph};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::TryRecvError;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_secs(10);
const RESCAN_INTERVAL: Duration = Duration::from_secs(60);
const FRAME_INTERVAL: Duration = Duration::from_millis(250);

pub fn art(stage: Stage, mood: Mood) -> Vec<&'static str> {
    match (stage, mood) {
        (Stage::Egg, Mood::Sleepy) => vec!["   ___   ", "  /   \\  ", " | z z | ", "  \\___/  "],
        (Stage::Egg, _) => vec!["   ___   ", "  / . \\  ", " |  .  | ", "  \\___/  "],
        (Stage::Baby, Mood::Sleepy) => vec!["  .---.  ", " ( -.- ) ", "  ( z )  ", "  '---'  "],
        (Stage::Baby, Mood::Hungry) => vec!["  .---.  ", " ( o.o ) ", "  ( O )  ", "  '---'  "],
        (Stage::Baby, Mood::Happy) => vec!["  .---.  ", " ( ^.^ ) ", "  ( v )  ", "  '---'  "],
        (Stage::Baby, Mood::Content) => vec!["  .---.  ", " ( o.o ) ", "  ( - )  ", "  '---'  "],
        (Stage::Adult, Mood::Sleepy) => vec![
            " /\\___/\\ ",
            "( -   - )",
            " ( zzz ) ",
            " /|   |\\ ",
            "  '---'  ",
        ],
        (Stage::Adult, Mood::Hungry) => vec![
            " /\\___/\\ ",
            "( o   o )",
            " (  O  ) ",
            " /|   |\\ ",
            "  '---'  ",
        ],
        (Stage::Adult, Mood::Happy) => vec![
            " /\\___/\\ ",
            "( ^   ^ )",
            " (  w  ) ",
            " /|   |\\ ",
            "  '---'  ",
        ],
        (Stage::Adult, Mood::Content) => vec![
            " /\\___/\\ ",
            "( o   o )",
            " (  -  ) ",
            " /|   |\\ ",
            "  '---'  ",
        ],
    }
}

pub struct View<'a> {
    pub pet: &'a Pet,
    pub repo_count: usize,
    pub author: &'a str,
    pub message: &'a str,
    /// Passed curriculum modules.
    pub badges: &'a [u8],
}

pub fn draw(f: &mut Frame, v: &View) {
    let rows = Layout::vertical([
        Constraint::Length(8),
        Constraint::Length(4),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .split(f.area());

    let pet = v.pet;
    let mut lines: Vec<Line> = art(pet.stage(), pet.mood())
        .into_iter()
        .map(|s| Line::from(s).centered())
        .collect();
    let mut status = format!("{:?} / {:?}", pet.stage(), pet.mood());
    if !v.badges.is_empty() {
        let b: Vec<String> = v.badges.iter().map(|m| format!("[{m}]")).collect();
        status.push_str(&format!("   Badges: {}", b.join(" ")));
    }
    lines.push(Line::from(status).centered());
    f.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(format!(
            " Gitagochi - watching {} repo{} ",
            v.repo_count,
            if v.repo_count == 1 { "" } else { "s" }
        ))),
        rows[0],
    );

    let bars =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(rows[1]);
    let gauge = |title: &str, val: f64, color: Color| {
        Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title.to_string()),
            )
            .gauge_style(Style::default().fg(color))
            .ratio((val / 100.0).clamp(0.0, 1.0))
            .label(format!("{:.0}%", val))
    };
    f.render_widget(gauge(" Hunger ", pet.hunger, Color::Yellow), bars[0]);
    f.render_widget(gauge(" Health ", pet.health, Color::Green), bars[1]);

    let items: Vec<ListItem> = if pet.recent.is_empty() {
        vec![ListItem::new(format!(
            "No commits fed yet (author: {})",
            v.author
        ))]
    } else {
        pet.recent
            .iter()
            .map(|c| {
                let id: String = c.id.chars().take(7).collect();
                if c.repo.is_empty() {
                    ListItem::new(format!("{id} {}", c.summary))
                } else {
                    ListItem::new(format!("[{}] {id} {}", c.repo, c.summary))
                }
            })
            .collect()
    };
    f.render_widget(
        List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Recent commits fed ({} total) ", pet.total_fed)),
        ),
        rows[2],
    );

    let hint = format!("q: quit  r: refresh  l: learn  {}", v.message);
    f.render_widget(Paragraph::new(hint), rows[3]);
}

/// Inputs for the interactive session.
pub struct Session {
    pub paths: Vec<PathBuf>,
    pub author: String,
    pub save_file: PathBuf,
    pub initial_warning: Option<String>,
    pub progress: Progress,
    pub progress_file: PathBuf,
    pub grade_env: GradeEnv,
    pub default_project: PathBuf,
}

/// Run the interactive loop. Terminal is restored on return and on panic
/// (`ratatui::init` installs a panic hook that restores it).
pub fn run(session: Session, resolved: Resolved, mut pet: Pet) -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal, session, resolved, &mut pet);
    ratatui::restore();
    result
}

fn pet_status(pet: &Pet) -> String {
    format!(
        " Pet: {:?} / {:?}   hunger {:.0}%   health {:.0}%",
        pet.stage(),
        pet.mood(),
        pet.hunger,
        pet.health
    )
}

fn event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    session: Session,
    resolved: Resolved,
    pet: &mut Pet,
) -> io::Result<()> {
    let author = session.author.as_str();
    let save_file = session.save_file.as_path();
    let mut paths = session.paths.clone();
    let mut progress = session.progress;
    let mut learn = Learn::new(session.default_project.clone());
    let mut job: Option<Job> = None;
    let mut repos = resolved.repos;
    let mut warnings = resolved.warnings;
    warnings.extend(session.initial_warning.clone());
    let mut last_scan = Instant::now();
    let mut message = String::new();
    let mut last_poll: Option<Instant> = None;
    let mut force_scan = false;
    let mut dirty = false;
    loop {
        let due = last_poll.is_none_or(|t| t.elapsed() >= POLL_INTERVAL);
        if due {
            if force_scan || last_scan.elapsed() >= RESCAN_INTERVAL {
                let r = repos::resolve(&paths);
                repos = r.repos;
                warnings = r.warnings;
                warnings.extend(session.initial_warning.clone());
                last_scan = Instant::now();
                force_scan = false;
            }
            refresh(pet, &repos, author, &warnings, &mut message);
            dirty = true;
            last_poll = Some(Instant::now());
        } else {
            pet.tick(now_unix());
        }
        if dirty {
            if let Err(e) = store::save(save_file, pet) {
                message = format!("save failed: {e}");
            }
            dirty = false;
        }
        if let Some(j) = &job {
            let got = match j.rx.try_recv() {
                Ok(r) => Some(r),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => {
                    Some(Err(GradeError::Io("the check stopped unexpectedly".into())))
                }
            };
            if let Some(r) = got {
                let m = j.module;
                if r.as_ref().is_ok_and(|rep| rep.passed()) {
                    progress.record_project_pass(m);
                    save_progress(&session.progress_file, &progress, &mut learn);
                }
                learn.check_done(m, r);
                job = None;
            }
        }
        let badges = progress.badges();
        terminal.draw(|f| {
            if learn.nav == Nav::Pet {
                draw(
                    f,
                    &View {
                        pet,
                        repo_count: repos.len(),
                        author,
                        message: &message,
                        badges: &badges,
                    },
                )
            } else {
                learn::draw(f, &learn, &progress, &pet_status(pet))
            }
        })?;
        if event::poll(FRAME_INTERVAL)?
            && let Event::Key(k) = event::read()?
            && k.kind == KeyEventKind::Press
        {
            if k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL) {
                break;
            }
            match learn.handle_key(k.code, &mut progress, quiz::clock_seed()) {
                Action::None => {}
                Action::Quit => break,
                Action::Refresh => {
                    last_poll = None;
                    force_scan = true;
                }
                Action::SaveProgress => {
                    save_progress(&session.progress_file, &progress, &mut learn)
                }
                Action::StartCheck(m) => {
                    if let (None, Some(p)) = (&job, &progress.project) {
                        job = Some(grade::spawn(p.clone(), m, session.grade_env.clone()));
                        learn.running = Some(m);
                    }
                }
                Action::Scaffold(dir) => match scaffold::create(&dir) {
                    Ok(()) => {
                        let dir = std::path::absolute(&dir).unwrap_or(dir);
                        progress.project = Some(dir.clone());
                        save_progress(&session.progress_file, &progress, &mut learn);
                        if !paths.contains(&dir) {
                            paths.push(dir.clone());
                        }
                        force_scan = true;
                        last_poll = None;
                        learn.setup_done(format!(
                            "Project created in {}; it is watched, so commits there feed your pet",
                            dir.display()
                        ));
                    }
                    Err(e) => learn.setup_failed(e.to_string()),
                },
            }
        }
    }
    if let Some(j) = job {
        j.cancel_and_join();
    }
    let _ = store::save(save_file, pet);
    Ok(())
}

fn save_progress(path: &Path, progress: &Progress, learn: &mut Learn) {
    if let Err(e) = store::save_progress(path, progress) {
        learn.note = format!("saving progress failed: {e}");
    }
}

fn short_error(e: &str) -> String {
    let first = e.lines().find(|l| !l.trim().is_empty()).unwrap_or("error");
    let first: String = first.chars().filter(|c| !c.is_control()).collect();
    if first.chars().count() > 80 {
        let cut: String = first.chars().take(77).collect();
        format!("sync error: {cut}...")
    } else {
        format!("sync error: {first}")
    }
}

fn refresh(pet: &mut Pet, repos: &[Repo], author: &str, warnings: &[String], message: &mut String) {
    let report = sync_all(pet, repos, author, now_unix());
    let mut parts = Vec::new();
    if report.fed > 0 {
        parts.push(format!("fed {} new commit(s)!", report.fed));
    }
    if let Some(e) = report.errors.first() {
        parts.push(short_error(e));
    } else if repos.is_empty() {
        parts.push("no repos watched yet - press l for the Rust academy".into());
    } else if let Some(w) = warnings.first() {
        parts.push(short_error(w).replacen("sync error", "warning", 1));
    }
    *message = parts.join("  ");
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn lrn_29_notice_when_no_repos() {
        let mut pet = Pet::new(1000);
        let mut msg = String::new();
        refresh(&mut pet, &[], "", &[], &mut msg);
        assert!(msg.contains("no repos watched yet"), "{msg}");
    }

    #[test]
    fn lrn_21_a_badge_shown_pet_unchanged() {
        let pet = Pet::new(1000);
        let before = pet.clone();
        let mut progress = Progress::default();
        progress.record_quiz(1, 100);
        progress.record_project_pass(1);
        let badges = progress.badges();
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
            .map(|p| buf[p].symbol().to_string())
            .collect();
        assert!(text.contains("Badges: [1]"), "{text}");
        assert!(text.contains("l: learn"));
        assert_eq!(pet, before);
    }

    #[test]
    fn errors_are_one_short_line() {
        let s = short_error(&format!("git failed: {}\nsecond\x1b line", "x".repeat(200)));
        assert!(!s.contains('\n'));
        assert!(s.chars().count() <= 93);
        assert!(s.starts_with("sync error: "));
    }
}
