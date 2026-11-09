//! Curriculum types, the shipped module list and lookups.

use super::content::{m01, m02};

/// Objective id: (module number, objective number), both 1-based.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjId(pub u8, pub u8);

impl std::fmt::Display for ObjId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Topic {
    Tooling,
    Basics,
    Ownership,
    Types,
    Errors,
    Collections,
    Traits,
    Modules,
    Lifetimes,
    Idioms,
}

impl Topic {
    pub const ALL: [Topic; 10] = [
        Topic::Tooling,
        Topic::Basics,
        Topic::Ownership,
        Topic::Types,
        Topic::Errors,
        Topic::Collections,
        Topic::Traits,
        Topic::Modules,
        Topic::Lifetimes,
        Topic::Idioms,
    ];
}

#[derive(Debug, Clone, Copy)]
pub struct Objective {
    pub id: ObjId,
    pub text: &'static str,
    pub section: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct Section {
    pub title: &'static str,
    pub body: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    MultipleChoice,
    SpotTheBug,
}

#[derive(Debug, Clone, Copy)]
pub struct Item {
    pub kind: ItemKind,
    pub prompt: &'static str,
    pub choices: &'static [&'static str],
    pub answer: usize,
    pub tags: &'static [ObjId],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckKind {
    Build,
    Fmt,
    Clippy,
    Test(&'static str),
}

impl CheckKind {
    pub fn name(&self) -> &'static str {
        match self {
            CheckKind::Build => "build",
            CheckKind::Fmt => "fmt",
            CheckKind::Clippy => "clippy",
            CheckKind::Test(n) => n,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Check {
    pub kind: CheckKind,
    pub tags: &'static [ObjId],
}

#[derive(Debug, Clone, Copy)]
pub struct Module {
    pub title: &'static str,
    pub topic: Topic,
    pub objectives: &'static [Objective],
    pub sections: &'static [Section],
    pub read_more: &'static [&'static str],
    pub brief: &'static str,
    pub quiz: &'static [Item],
    pub checks: &'static [Check],
    pub hidden_tests: &'static str,
}

/// The shipped curriculum; module number = index + 1.
pub static MODULES: &[Module] = &[m01::MODULE, m02::MODULE];

/// Look up an objective in the shipped curriculum.
pub fn objective(id: ObjId) -> Option<&'static Objective> {
    objective_in(MODULES, id)
}

pub fn objective_in(modules: &[Module], id: ObjId) -> Option<&Objective> {
    modules
        .get(usize::from(id.0).checked_sub(1)?)?
        .objectives
        .iter()
        .find(|o| o.id == id)
}

/// Find a lesson section of module `m` (1-based) by title.
pub fn section(m: u8, title: &str) -> Option<&'static Section> {
    MODULES
        .get(usize::from(m).checked_sub(1)?)?
        .sections
        .iter()
        .find(|s| s.title == title)
}

/// Checks of modules `1..=n` in module order, with Build/Fmt/Clippy once.
/// Each entry is (module number, check).
pub fn checks_upto(modules: &[Module], n: usize) -> Vec<(u8, Check)> {
    let mut out: Vec<(u8, Check)> = Vec::new();
    for (i, m) in modules.iter().take(n).enumerate() {
        for c in m.checks {
            let dup =
                !matches!(c.kind, CheckKind::Test(_)) && out.iter().any(|(_, o)| o.kind == c.kind);
            if !dup {
                out.push((i as u8 + 1, *c));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_all_is_exhaustive() {
        for t in Topic::ALL {
            match t {
                Topic::Tooling
                | Topic::Basics
                | Topic::Ownership
                | Topic::Types
                | Topic::Errors
                | Topic::Collections
                | Topic::Traits
                | Topic::Modules
                | Topic::Lifetimes
                | Topic::Idioms => {}
            }
        }
        assert_eq!(Topic::ALL.len(), 10);
    }

    const T: &[ObjId] = &[ObjId(1, 1)];

    fn synthetic(checks: &'static [Check]) -> Module {
        Module {
            title: "m",
            topic: Topic::Basics,
            objectives: &[],
            sections: &[],
            read_more: &[],
            brief: "",
            quiz: &[],
            checks,
            hidden_tests: "",
        }
    }

    #[test]
    fn lrn_16_checks_upto_includes_earlier() {
        let mods = [
            synthetic(&[
                Check {
                    kind: CheckKind::Build,
                    tags: T,
                },
                Check {
                    kind: CheckKind::Test("m01_a"),
                    tags: T,
                },
                Check {
                    kind: CheckKind::Fmt,
                    tags: T,
                },
                Check {
                    kind: CheckKind::Clippy,
                    tags: T,
                },
            ]),
            synthetic(&[Check {
                kind: CheckKind::Test("m02_a"),
                tags: T,
            }]),
            synthetic(&[
                Check {
                    kind: CheckKind::Test("m03_a"),
                    tags: T,
                },
                Check {
                    kind: CheckKind::Clippy,
                    tags: T,
                },
            ]),
            synthetic(&[Check {
                kind: CheckKind::Test("m04_a"),
                tags: T,
            }]),
        ];
        let got: Vec<(u8, CheckKind)> = checks_upto(&mods, 3)
            .into_iter()
            .map(|(m, ch)| (m, ch.kind))
            .collect();
        let want = vec![
            (1, CheckKind::Build),
            (1, CheckKind::Test("m01_a")),
            (1, CheckKind::Fmt),
            (1, CheckKind::Clippy),
            (2, CheckKind::Test("m02_a")),
            (3, CheckKind::Test("m03_a")),
        ];
        assert_eq!(got, want);
    }
}
