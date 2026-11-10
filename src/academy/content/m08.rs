use crate::academy::curriculum::{
    Check, CheckKind, Item, ItemKind, Module, ObjId, Objective, Section, Topic,
};

const O1: ObjId = ObjId(8, 1);
const O2: ObjId = ObjId(8, 2);
const O3: ObjId = ObjId(8, 3);

pub const MODULE: Module = Module {
    title: "Modules, visibility and testing",
    topic: Topic::Modules,
    objectives: &[
        Objective {
            id: O1,
            text: "Split a crate into modules and files with mod",
            section: "Modules and files",
        },
        Objective {
            id: O2,
            text: "Control visibility with pub and re-export with pub use",
            section: "Visibility and re-exports",
        },
        Objective {
            id: O3,
            text: "Organise unit tests and integration tests",
            section: "Unit and integration tests",
        },
    ],
    sections: &[
        Section {
            title: "Modules and files",
            body: r#"A module is a named scope for items. `mod name;` in src/lib.rs tells the
compiler to load the module from src/name.rs (or src/name/mod.rs):

    // src/lib.rs
    pub mod parse;      // contents in src/parse.rs
    pub mod stats;      // contents in src/stats.rs

The file itself does not repeat `mod parse { .. }`; its whole content is the
module. A file that no `mod` declaration names is not compiled at all.

Paths name items through modules: `crate::parse::Commit` from anywhere in the
crate, `super::x` for the parent module, `self::x` for the current one.
`use` brings a path into scope so you can write the short name:

    // src/stats.rs
    use crate::parse::{Commit, field};

From outside the crate, paths start with the crate name:
`tracker::parse::Commit`."#,
        },
        Section {
            title: "Visibility and re-exports",
            body: r#"Everything is private to its module by default. `pub` makes an item visible to
the parent module and, if every module on the path is pub, to other crates.

    mod helpers { pub fn f() {} }   // private module: tracker::helpers::f is
                                     // not reachable from outside the crate
    pub mod stats { fn g() {} }     // g is private: error[E0603] from outside

For structs, `pub struct` does not make fields public; each field needs its
own `pub`. Enum variants are public whenever the enum is.

`pub use` re-exports an item under a new path. It is how a crate keeps old
paths working after moving code, and how it offers a short path to common
items:

    // src/lib.rs
    pub mod parse;
    pub use parse::{Commit, parse_log};   // tracker::Commit still works

Now both tracker::parse::Commit and tracker::Commit name the same type.
Plain `use` (without pub) only creates a private shortcut inside the module.
`pub(crate)` makes an item visible in the whole crate but not outside it."#,
        },
        Section {
            title: "Unit and integration tests",
            body: r#"Unit tests sit next to the code, in a #[cfg(test)] mod tests inside each
module file. Because they are inside the module, `use super::*;` gives them
access to private items too.

Integration tests are files in the tests/ folder at the package root (next to
src/). Each file is compiled as its own separate crate that uses your library
like any user would:

    // tests/report.rs
    use tracker::parse::parse_log;

    #[test]
    fn parses() {
        assert_eq!(parse_log("").unwrap().len(), 0);
    }

So integration tests can reach only `pub` items, by their public paths. They
need a library crate (src/lib.rs); a binary-only package cannot be tested this
way. They do not need #[cfg(test)], since the whole tests/ folder is built
only by cargo test.

    cargo test                 run unit, integration and doc tests
    cargo test --test report   run only tests/report.rs
    cargo test --lib           run only the unit tests

Test helpers can return Result and use `?` instead of unwrap:
`fn t() -> Result<(), ParseError> { parse_log("x|y|1|z")?; Ok(()) }`."#,
        },
    ],
    read_more: &[
        "https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html",
        "https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html",
        "https://doc.rust-lang.org/book/ch11-03-test-organization.html",
    ],
    brief: r#"Step 8: organise the crate. Move code out of src/lib.rs into two modules:

    src/parse.rs   pub mod parse: Commit, Kind, ParseError, field,
                   parse_commit, parse_line, parse_log, and Display for Commit
    src/stats.rs   pub mod stats: everything else (SECONDS_PER_DAY,
                   count_lines, day_of, longest_streak, authors, normalize,
                   commits_per_author, top_authors, streak, Stat,
                   TotalCommits, LongestStreak, report, most_common)

src/lib.rs declares both modules and uses `pub use` so every old path such as
tracker::parse_log and tracker::Commit keeps working.

Move each unit test into the module it tests, and add an integration test in
tests/ that uses tracker::parse and tracker::stats. Do not name it
tests/academy_m*.rs; those names are reserved for the grader.
Test, fmt, clippy, commit."#,
    quiz: &[
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Where does `pub mod stats;` in src/lib.rs load the module from?",
            choices: &[
                "src/stats.rs (or src/stats/mod.rs)",
                "tests/stats.rs",
                "Cargo.toml",
                "Nowhere; it declares an empty module",
            ],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "src/parse.rs exists, but `crate::parse::Commit` gives \"unresolved import\". Pick the fix.",
            choices: &[
                "Add `pub mod parse;` to src/lib.rs",
                "Add `mod parse { }` at the top of src/parse.rs",
                "Rename it to src/parse.mod",
                "Add parse to Cargo.toml",
            ],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Inside src/stats.rs, which path names Commit from src/parse.rs?",
            choices: &[
                "crate::parse::Commit",
                "tracker::Commit only",
                "super::super::Commit",
                "parse.Commit",
            ],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What does `pub use parse::Commit;` in src/lib.rs do?",
            choices: &[
                "Makes tracker::Commit another public path to parse::Commit",
                "Copies the Commit type",
                "Makes Commit's fields public",
                "Imports Commit only for lib.rs",
            ],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "Another crate gets E0603 (function `g` is private) calling tracker::stats::g, where src/stats.rs has `fn g() {}` and lib.rs has `pub mod stats;`. Pick the fix.",
            choices: &[
                "Write pub fn g() in src/stats.rs",
                "Write pub mod stats twice",
                "Add use stats::g; to lib.rs",
                "Move g into tests/",
            ],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Given `pub struct Fields { hash: String }`, can another crate read `f.hash`?",
            choices: &[
                "No: the field also needs pub",
                "Yes: pub struct makes fields public",
                "Only in tests",
                "Only with use",
            ],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What does `pub(crate) fn f()` allow?",
            choices: &[
                "Use anywhere in this crate, but not from other crates",
                "Use from any crate",
                "Use only in the same file",
                "Use only in tests",
            ],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Which items can an integration test in tests/ call?",
            choices: &[
                "Only pub items, by their public paths",
                "Any item, including private ones",
                "Only items in main.rs",
                "Only #[cfg(test)] items",
            ],
            answer: 0,
            tags: &[O3],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "A unit test inside src/stats.rs cannot find the private fn helper:\n\n    #[cfg(test)]\n    mod tests {\n        #[test]\n        fn t() { assert_eq!(helper(), 1); }\n    }\n\nPick the fix.",
            choices: &[
                "Add use super::*; inside mod tests",
                "Make the test module pub",
                "Move the test to tests/stats.rs",
                "Remove #[cfg(test)]",
            ],
            answer: 0,
            tags: &[O3],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Which command runs only the integration tests in tests/report.rs?",
            choices: &[
                "cargo test --test report",
                "cargo test --lib",
                "cargo run --test",
                "cargo build --tests",
            ],
            answer: 0,
            tags: &[O3],
        },
    ],
    checks: &[
        Check {
            kind: CheckKind::Test("m08_parse_module_path"),
            tags: &[O1, O2],
        },
        Check {
            kind: CheckKind::Test("m08_stats_module_path"),
            tags: &[O1, O2],
        },
    ],
    hidden_tests: include_str!("../../../curriculum/hidden/m08.rs"),
};
