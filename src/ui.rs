//! Terminal UI: renders from state and runs the event loop.

use crate::git::Repo;
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
use std::path::PathBuf;
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
    lines.push(Line::from(format!("{:?} / {:?}", pet.stage(), pet.mood())).centered());
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

    let hint = format!("q: quit  r: refresh  {}", v.message);
    f.render_widget(Paragraph::new(hint), rows[3]);
}

/// Inputs for the interactive session.
pub struct Session {
    pub paths: Vec<PathBuf>,
    pub author: String,
    pub save_file: PathBuf,
    pub initial_warning: Option<String>,
}

/// Run the interactive loop. Terminal is restored on return and on panic
/// (`ratatui::init` installs a panic hook that restores it).
pub fn run(session: Session, resolved: Resolved, mut pet: Pet) -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal, &session, resolved, &mut pet);
    ratatui::restore();
    result
}

fn event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    session: &Session,
    resolved: Resolved,
    pet: &mut Pet,
) -> io::Result<()> {
    let author = session.author.as_str();
    let save_file = session.save_file.as_path();
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
                let r = repos::resolve(&session.paths);
                repos = r.repos;
                warnings = r.warnings;
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
        terminal.draw(|f| {
            draw(
                f,
                &View {
                    pet,
                    repo_count: repos.len(),
                    author,
                    message: &message,
                },
            )
        })?;
        if event::poll(FRAME_INTERVAL)?
            && let Event::Key(k) = event::read()?
            && k.kind == KeyEventKind::Press
        {
            match k.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Char('r') => {
                    last_poll = None;
                    force_scan = true;
                }
                _ => {}
            }
        }
    }
    let _ = store::save(save_file, pet);
    Ok(())
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
    } else if let Some(w) = warnings.first() {
        parts.push(short_error(w).replacen("sync error", "warning", 1));
    }
    *message = parts.join("  ");
}

#[cfg(test)]
mod tests {
    use super::short_error;

    #[test]
    fn errors_are_one_short_line() {
        let s = short_error(&format!("git failed: {}\nsecond\x1b line", "x".repeat(200)));
        assert!(!s.contains('\n'));
        assert!(s.chars().count() <= 93);
        assert!(s.starts_with("sync error: "));
    }
}
