//! Terminal UI: renders from state and runs the event loop.

use crate::git::Repo;
use crate::pet::{Mood, Pet, Stage};
use crate::{now_unix, store, sync};
use ratatui::Frame;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, Paragraph};
use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_secs(10);
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
    pub repo_name: &'a str,
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
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Gitagochi - {} ", v.repo_name)),
        ),
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
                ListItem::new(format!("{id} {}", c.summary))
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

/// Run the interactive loop. Terminal is restored on return and on panic
/// (`ratatui::init` installs a panic hook that restores it).
pub fn run(repo: &Repo, author: &str, save_file: &Path, mut pet: Pet) -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal, repo, author, save_file, &mut pet);
    ratatui::restore();
    result
}

fn event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    repo: &Repo,
    author: &str,
    save_file: &Path,
    pet: &mut Pet,
) -> io::Result<()> {
    let repo_name = repo
        .root()
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "repo".into());
    let mut message = String::new();
    let mut last_poll: Option<Instant> = None;
    let mut dirty = false;
    loop {
        let due = last_poll.is_none_or(|t| t.elapsed() >= POLL_INTERVAL);
        if due {
            refresh(pet, repo, author, &mut message, &mut dirty);
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
                    repo_name: &repo_name,
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
                KeyCode::Char('r') => last_poll = None,
                _ => {}
            }
        }
    }
    let _ = store::save(save_file, pet);
    Ok(())
}

fn refresh(pet: &mut Pet, repo: &Repo, author: &str, message: &mut String, dirty: &mut bool) {
    match sync(pet, repo, author, now_unix()) {
        Ok(n) => {
            *message = if n > 0 {
                format!("fed {n} new commit(s)!")
            } else {
                String::new()
            };
            *dirty = true;
        }
        Err(e) => *message = format!("{e}"),
    }
}
