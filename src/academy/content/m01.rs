use crate::academy::curriculum::{
    Check, CheckKind, Item, ItemKind, Module, ObjId, Objective, Section, Topic,
};

const O1: ObjId = ObjId(1, 1);
const O2: ObjId = ObjId(1, 2);
const O3: ObjId = ObjId(1, 3);

pub const MODULE: Module = Module {
    title: "Cargo and tooling",
    topic: Topic::Tooling,
    objectives: &[
        Objective {
            id: O1,
            text: "Build, run and test a crate with a library and a binary using cargo",
            section: "Cargo projects",
        },
        Objective {
            id: O2,
            text: "Write unit tests with #[test] and assert_eq!",
            section: "Unit tests",
        },
        Objective {
            id: O3,
            text: "Keep code formatted with cargo fmt and lint-free with cargo clippy",
            section: "Formatting and linting",
        },
    ],
    sections: &[
        Section {
            title: "Cargo projects",
            body: r#"Cargo is Rust's build tool and package manager. A package is a folder with a
Cargo.toml manifest:

    [package]
    name = "tracker"
    version = "0.1.0"
    edition = "2024"

A package holds crates. If src/lib.rs exists, the package has a library crate
named after the package (here `tracker`). If src/main.rs exists, it also has a
binary crate of the same name. The binary uses the library like any other
crate, by its name:

    // src/main.rs
    fn main() {
        println!("{}", tracker::count_lines("a\nb\n"));
    }

Items in the library must be marked `pub` to be used from main.rs or from
tests in the tests/ folder.

The everyday commands, run in the package folder:

    cargo build    compile everything (output goes to target/debug)
    cargo run      build, then run the binary
    cargo test     build and run every test
    cargo check    type-check without producing a binary (fast)

`cargo build` prints each error with a code such as error[E0308];
`rustc --explain E0308` describes it.

The target/ folder holds build output and should not be committed; put
`/target` in .gitignore."#,
        },
        Section {
            title: "Unit tests",
            body: r#"A test is a function marked with the #[test] attribute. It passes if it returns
without panicking. A function without #[test] is never run as a test, whatever
its name. Unit tests usually live in a `tests` module at the bottom of the file
they test:

    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn adds() {
            assert_eq!(add(2, 3), 5);
        }
    }

#[cfg(test)] means the module is compiled only by `cargo test`, not by
`cargo build`. `use super::*;` brings the parent module's items into scope,
including private ones.

The assertion macros panic when they fail, which fails the test:

    assert!(cond)           fails if cond is false
    assert_eq!(left, right) fails if left != right, printing both values
    assert_ne!(left, right) fails if left == right

assert_eq! needs both sides to be comparable with == and printable with {:?}.

`cargo test name` runs only tests whose name contains `name`."#,
        },
        Section {
            title: "Formatting and linting",
            body: r#"rustfmt formats code in the one standard Rust style:

    cargo fmt          rewrite files in place
    cargo fmt --check  change nothing; fail and show a diff if any file
                       is not formatted

Clippy is the official linter. It reports code that compiles but is
unidiomatic, slow or likely wrong:

    cargo clippy -- -D warnings

`-D warnings` turns every warning into an error, so the command fails if any
lint fires. Each lint has a name such as clippy::needless_return. For example,
clippy flags an explicit `return` on the last expression of a function,
because the last expression (without a semicolon) is already the result:

    fn one() -> u8 {
        return 1; // clippy::needless_return
    }

    fn one() -> u8 {
        1 // fine
    }

Both tools are rustup components (`rustup component add rustfmt clippy`).
In this course every project check runs `cargo fmt --check` and
`cargo clippy -- -D warnings` on your last commit, so run both before you
commit."#,
        },
    ],
    read_more: &[
        "https://doc.rust-lang.org/book/ch01-03-hello-cargo.html",
        "https://doc.rust-lang.org/book/ch11-01-writing-tests.html",
        "https://doc.rust-lang.org/book/appendix-04-useful-development-tools.html",
    ],
    brief: r#"Over the ten modules you build `tracker`, a command-line commit tracker that
reads a log of commits and reports stats, ending with a tiny pet of your own.

Set up the project with `p` on the learn screen: Gitagochi creates a cargo
package named `tracker` (src/lib.rs and src/main.rs) in the folder you choose,
runs `git init`, and watches it, so every commit you make there feeds your pet.
Keep the package name `tracker` and the lib + bin layout for the whole course.
Gitagochi needs the Rust toolchain on PATH to grade (https://rustup.rs);
edition 2024 needs Rust 1.85 or newer (`rustup update stable`).

The log has one commit per line, fields separated by `|`:

    hash|author|unix_seconds|summary

Make one from any repo with:

    git log --format=%h|%ae|%ct|%s --output=log.txt

Step 1. In src/lib.rs add

    pub fn count_lines(log: &str) -> usize

returning the number of non-blank lines in `log` (lines that are empty or hold
only spaces are not commits). In src/main.rs read log.txt and print the count.
Add a unit test of your own.

Run `cargo test`, `cargo fmt` and `cargo clippy -- -D warnings`, then commit.
The project check grades your last commit, not uncommitted edits."#,
    quiz: &[
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "A package named tracker has src/lib.rs and src/main.rs. How does main.rs call the library's `pub fn count_lines`?",
            choices: &[
                "tracker::count_lines(..)",
                "lib::count_lines(..)",
                "include!(\"lib.rs\") first",
                "It cannot; a package has one crate",
            ],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Which command builds the project and runs all its tests?",
            choices: &["cargo run", "cargo build", "cargo test", "cargo check"],
            answer: 2,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Which folder holds build output and should be listed in .gitignore?",
            choices: &["src/", "target/", "tests/", "Cargo.toml"],
            answer: 1,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What does #[cfg(test)] on a module do?",
            choices: &[
                "Runs the module's code before main",
                "Compiles the module only when running cargo test",
                "Marks every function in it as a test",
                "Makes the module public",
            ],
            answer: 1,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "When does a #[test] function pass?",
            choices: &[
                "When it returns without panicking",
                "Only when it calls assert_eq!",
                "When it prints \"ok\"",
                "When it returns true",
            ],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "This test never runs under cargo test:\n\n    #[cfg(test)]\n    mod tests {\n        use super::*;\n        fn adds() { assert_eq!(add(2, 3), 5); }\n    }\n\nPick the fix.",
            choices: &[
                "Add #[test] above fn adds",
                "Rename it to test_adds",
                "Make it pub fn adds",
                "Replace assert_eq! with assert!",
            ],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What does `cargo fmt --check` do?",
            choices: &[
                "Formats every file in place",
                "Fails and shows a diff if any file is not formatted, changing nothing",
                "Runs clippy",
                "Checks that the code compiles",
            ],
            answer: 1,
            tags: &[O3],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What does `-D warnings` do in `cargo clippy -- -D warnings`?",
            choices: &[
                "Hides all warnings",
                "Shows warnings only in debug builds",
                "Turns every warning into an error so the command fails",
                "Deletes code that causes warnings",
            ],
            answer: 2,
            tags: &[O3],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "Clippy rejects this with clippy::needless_return:\n\n    pub fn one() -> u8 {\n        return 1;\n    }\n\nPick the fix.",
            choices: &[
                "pub fn one() -> u8 { 1 }",
                "pub fn one() -> u8 { return 1 }",
                "pub fn one() -> i32 { return 1; }",
                "#[test] pub fn one() -> u8 { return 1; }",
            ],
            answer: 0,
            tags: &[O3],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "src/main.rs fails to compile with \"function `helper` is private\" when calling tracker::helper(). Pick the fix.",
            choices: &[
                "Mark helper as pub in src/lib.rs",
                "Rename main.rs to lib.rs",
                "Add #[test] to helper",
                "Run cargo build twice",
            ],
            answer: 0,
            tags: &[O1],
        },
    ],
    checks: &[
        Check {
            kind: CheckKind::Build,
            tags: &[O1],
        },
        Check {
            kind: CheckKind::Test("m01_counts_non_empty_lines"),
            tags: &[O1],
        },
        Check {
            kind: CheckKind::Test("m01_empty_log_is_zero"),
            tags: &[O1],
        },
        Check {
            kind: CheckKind::Fmt,
            tags: &[O3],
        },
        Check {
            kind: CheckKind::Clippy,
            tags: &[O3],
        },
    ],
    hidden_tests: include_str!("../../../curriculum/hidden/m01.rs"),
};
