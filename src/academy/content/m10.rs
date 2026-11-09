use crate::academy::curriculum::{
    Check, CheckKind, Item, ItemKind, Module, ObjId, Objective, Section, Topic,
};

const O1: ObjId = ObjId(10, 1);
const O2: ObjId = ObjId(10, 2);
const O3: ObjId = ObjId(10, 3);

pub const MODULE: Module = Module {
    title: "Idiomatic Rust",
    topic: Topic::Idioms,
    objectives: &[
        Objective {
            id: O1,
            text: "Implement the std traits Default, Display, FromStr and Error",
            section: "Standard trait implementations",
        },
        Objective {
            id: O2,
            text: "Encapsulate state behind methods so illegal states cannot be built",
            section: "Encapsulation",
        },
        Objective {
            id: O3,
            text: "Write clippy-clean idioms: borrowed parameters, iterators, saturating math",
            section: "Clippy-clean idioms",
        },
    ],
    sections: &[
        Section {
            title: "Standard trait implementations",
            body: r#"Implementing std traits makes your types work with the rest of Rust.

Default gives a type a standard starting value, used by `T::default()` and
`..Default::default()`:

    impl Default for Pet {
        fn default() -> Self { Pet { hunger: 50 } }
    }

Display (module 7) gives `{}` and to_string().

FromStr lets `text.parse::<T>()` build your type. It names its error type:

    use std::str::FromStr;
    impl FromStr for Commit {
        type Err = ParseError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            parse_line(s, 1)
        }
    }
    let c: Commit = "a1|ann|5|x".parse()?;

std::error::Error marks a type as an error. It needs Debug and Display; the
impl itself can be empty:

    impl fmt::Display for ParseError { .. }
    impl std::error::Error for ParseError {}

Then your error fits anywhere a `&dyn std::error::Error` or
`Box<dyn std::error::Error>` is expected, for example as main's error type:
`fn main() -> Result<(), Box<dyn std::error::Error>>`."#,
        },
        Section {
            title: "Encapsulation",
            body: r#"Keep fields private when the type has rules, and expose methods that keep the
rules. Then no code outside the module can put the value in a bad state.

    pub struct Pet {
        hunger: u8,               // private: 0..=100 is guaranteed by the methods
    }

    impl Pet {
        pub fn hunger(&self) -> u8 { self.hunger }      // a getter
        pub fn feed(&mut self, commits: usize) { .. }   // keeps 0..=100
    }

Outside the module, `Pet { hunger: 250 }` and `pet.hunger = 250` are errors
(E0451 field is private / E0616), so the only way to change hunger is through
feed and the other methods.

Choose types that cannot hold invalid values: an enum instead of a string
for a fixed set of states (Mood::Happy, not "happy"), u8 instead of i32 for a
value that is never negative, Option instead of a magic value such as -1.
"Make illegal states unrepresentable": if a value cannot be built, no code
needs to check for it.

A struct with private fields has to offer a constructor (new or Default),
since other modules cannot write the struct literal."#,
        },
        Section {
            title: "Clippy-clean idioms",
            body: r#"Habits that keep `cargo clippy -- -D warnings` quiet and code clear:

Borrow, do not take ownership, when you only read: `&str` not `String`,
`&[T]` not `Vec<T>` or `&Vec<T>` (clippy::ptr_arg, clippy::needless_pass_by_value
is pedantic but the habit is good).

Use iterators instead of index loops: `for x in &v` rather than
`for i in 0..v.len() { v[i] }` (clippy::needless_range_loop).

Use saturating (or checked) arithmetic where overflow is possible:

    let h: u8 = 200;
    h.saturating_add(100)    // 255, the max, instead of an overflow panic
    h.saturating_sub(250)    // 0 instead of a panic
    a.min(100)               // cap a value

`checked_add` returns None on overflow. Converting a big integer down safely:
`u8::try_from(n).unwrap_or(u8::MAX)`.

Other common lints: needless_return (module 1), `x.len() == 0` should be
`x.is_empty()` (clippy::len_zero), `if x == true` should be `if x`
(clippy::bool_comparison), and new() without Default should also implement
Default (clippy::new_without_default).

Run `cargo fmt` and `cargo clippy --all-targets -- -D warnings` before every
commit; this module's check runs both again."#,
        },
    ],
    read_more: &[
        "https://doc.rust-lang.org/book/ch17-01-what-is-oo.html",
        "https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html",
        "https://doc.rust-lang.org/book/appendix-03-derivable-traits.html",
    ],
    brief: r#"Step 10: your own pet. Add src/pet.rs as `pub mod pet` with:

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Mood { Happy, Hungry, Sleeping }

Display for Mood prints "happy", "hungry" or "sleeping".

    pub struct Pet { hunger: u8 }   // private field, always 0..=100

    impl Default for Pet      // hunger 50
    impl Pet {
        pub fn hunger(&self) -> u8
        pub fn feed(&mut self, commits: usize)   // each commit: hunger - 10, not below 0
        pub fn starve(&mut self, hours: u32)     // each hour: hunger + 10, not above 100
        pub fn mood(&self) -> Mood               // < 30 Happy, > 70 Hungry, else Sleeping
    }

feed and starve must never panic, even for huge values.

Also implement Display and std::error::Error for ParseError (Display shows the
line number, e.g. "line 3: missing field"), and FromStr for Commit
(Err = ParseError; "line".parse::<Commit>() treats it as line 1).

In main, feed a Pet one meal per commit in the log and print its mood.
Run cargo fmt and cargo clippy --all-targets -- -D warnings, then commit.
That completes the tracker."#,
    quiz: &[
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Which trait makes `\"a1|ann|5|x\".parse::<Commit>()` work?",
            choices: &["FromStr", "Display", "Default", "From<i64>"],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What does std::error::Error require of a type?",
            choices: &[
                "Debug and Display",
                "Clone and Copy",
                "Default",
                "Nothing; it is derived",
            ],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "This does not compile (missing `Err` in implementation):\n\n    impl FromStr for Commit {\n        fn from_str(s: &str) -> Result<Self, ParseError> { parse_line(s, 1) }\n    }\n\nPick the fix.",
            choices: &[
                "Add `type Err = ParseError;` and return Result<Self, Self::Err>",
                "Add #[derive(FromStr)]",
                "Rename from_str to parse",
                "Make from_str pub",
            ],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What is `Pet::default()` for after `impl Default for Pet`?",
            choices: &[
                "A standard starting value for the type",
                "Resets every Pet in the program",
                "Prints the pet",
                "It panics unless derived",
            ],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Why keep `hunger` private in `pub struct Pet { hunger: u8 }`?",
            choices: &[
                "Only Pet's methods can change it, so it always stays in 0..=100",
                "Private fields are faster",
                "pub fields cannot be u8",
                "So that Pet can derive Copy",
            ],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Which type best stores a pet's mood from a fixed set of three?",
            choices: &["An enum Mood", "A String", "An i32 code", "A bool"],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "Another module writes `let p = Pet { hunger: 250 };` and gets E0451 (field is private). The goal is a valid pet. Pick the fix.",
            choices: &[
                "Use Pet::default() and the pet's methods",
                "Make hunger pub so 250 is allowed",
                "Change hunger to i32",
                "Use unsafe",
            ],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What is `200u8.saturating_add(100)`?",
            choices: &["255", "44", "300", "It panics"],
            answer: 0,
            tags: &[O3],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "Clippy flags clippy::needless_range_loop:\n\n    for i in 0..commits.len() {\n        total += commits[i].time;\n    }\n\nPick the fix.",
            choices: &[
                "for c in commits { total += c.time; }",
                "for i in 0..=commits.len() { .. }",
                "while i < commits.len() { .. } without changing i",
                "Add #[allow(warnings)] to the crate",
            ],
            answer: 0,
            tags: &[O3],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "Clippy flags clippy::len_zero:\n\n    if names.len() == 0 { return None; }\n\nPick the fix.",
            choices: &[
                "if names.is_empty() { return None; }",
                "if names.len() < 1 { return None; }",
                "if names == 0 { return None; }",
                "if !names.len() { return None; }",
            ],
            answer: 0,
            tags: &[O3],
        },
    ],
    checks: &[
        Check {
            kind: CheckKind::Test("m10_pet_default_and_feed_saturates"),
            tags: &[O1, O2, O3],
        },
        Check {
            kind: CheckKind::Test("m10_mood_display"),
            tags: &[O1, O2],
        },
        Check {
            kind: CheckKind::Test("m10_commit_from_str"),
            tags: &[O1],
        },
        Check {
            kind: CheckKind::Test("m10_parse_error_is_std_error"),
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
    hidden_tests: include_str!("../../../curriculum/hidden/m10.rs"),
};
