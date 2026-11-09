use crate::academy::curriculum::{
    Check, CheckKind, Item, ItemKind, Module, ObjId, Objective, Section, Topic,
};

const O1: ObjId = ObjId(4, 1);
const O2: ObjId = ObjId(4, 2);
const O3: ObjId = ObjId(4, 3);

pub const MODULE: Module = Module {
    title: "Structs, enums and pattern matching",
    topic: Topic::Types,
    objectives: &[
        Objective {
            id: O1,
            text: "Define structs with impl blocks and methods",
            section: "Structs and methods",
        },
        Objective {
            id: O2,
            text: "Model choices with enums and absent values with Option",
            section: "Enums and Option",
        },
        Objective {
            id: O3,
            text: "Take values apart with match, if let and let else",
            section: "Pattern matching",
        },
    ],
    sections: &[
        Section {
            title: "Structs and methods",
            body: r#"A struct groups named fields. `pub` on the struct and on each field makes them
usable from other crates (such as your tests):

    #[derive(Debug, Clone, PartialEq)]
    pub struct Commit {
        pub hash: String,
        pub author: String,
        pub time: i64,
        pub summary: String,
    }

    let c = Commit {
        hash: "a1".to_string(),
        author: "ann@x.io".to_string(),
        time: 100,
        summary: "fix crash".to_string(),
    };
    println!("{}", c.author);

A struct literal must give every field. `#[derive(...)]` generates common
trait impls: Debug lets you print with {:?}, Clone adds .clone(), PartialEq
adds == (needed by assert_eq! and by comparing in tests).

Methods live in an `impl` block. A method's first parameter is the receiver:
`&self` reads the value, `&mut self` changes it, `self` consumes it. A function
in the impl without a receiver is an associated function, called with `::`:

    impl Commit {
        pub fn is_by(&self, who: &str) -> bool {
            self.author == who
        }
        pub fn new(hash: &str) -> Commit { ... }   // Commit::new("a1")
    }

    c.is_by("ann@x.io")   // method call, &self is passed for you

Inside an impl, `Self` is another name for the type."#,
        },
        Section {
            title: "Enums and Option",
            body: r#"An enum is a type whose value is exactly one of its variants:

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Kind {
        Feature,
        Fix,
        Docs,
        Other,
    }

    let k = Kind::Fix;

Variants can carry data: `enum Shape { Circle(f64), Rect { w: f64, h: f64 } }`.

Rust has no null. A value that may be absent is an `Option<T>`, an enum from
the standard library:

    enum Option<T> {
        Some(T),
        None,
    }

    fn first_char(s: &str) -> Option<char> { s.chars().next() }

You cannot use an Option<T> as a T; you must handle None first (next
section). Useful methods: `opt.is_some()`, `opt.is_none()`,
`opt.unwrap_or(default)`, and `text.parse::<i64>().ok()`, which turns a parse
result into Some(number) or None.

`s.starts_with("feat")` tests a prefix and returns bool."#,
        },
        Section {
            title: "Pattern matching",
            body: r#"`match` compares a value against patterns, top to bottom, and runs the first
arm that fits. It must be exhaustive: every possible value needs an arm, and
`_` matches anything.

    match k {
        Kind::Feature => "new",
        Kind::Fix => "fixed",
        _ => "other",
    }

Patterns bind the data inside variants:

    match first_char(s) {
        Some(c) => println!("starts with {c}"),
        None => println!("empty"),
    }

A guard adds a condition to an arm:

    match summary {
        s if s.starts_with("fix") => Kind::Fix,
        _ => Kind::Other,
    }

`if let` handles one pattern and ignores the rest:

    if let Some(c) = first_char(s) { println!("{c}"); }

`let ... else` binds a pattern or leaves the function:

    let (Some(a), Some(b)) = (parts.next(), parts.next()) else {
        return None;
    };
    // `a` and `b` are &str from here on

To split a line into a fixed number of fields, `splitn(n, '|')` stops after
n pieces, so the last piece keeps any further `|`:

    let mut parts = "a|b|c|d|e".splitn(4, '|');
    // pieces: "a", "b", "c", "d|e"

Inside a function that returns an Option, `?` after an Option is shorthand
for "take the value out of Some, or return None right now". Clippy
(clippy::question_mark) asks for it instead of a let-else that only returns
None:

    let time: i64 = text.parse::<i64>().ok()?;

Forgetting a variant in a match is error[E0004] non-exhaustive patterns."#,
        },
    ],
    read_more: &[
        "https://doc.rust-lang.org/book/ch05-00-structs.html",
        "https://doc.rust-lang.org/book/ch06-00-enums.html",
        "https://doc.rust-lang.org/book/ch19-00-patterns.html",
    ],
    brief: r#"Step 4: a Commit type. Add to src/lib.rs:

    #[derive(Debug, Clone, PartialEq)]
    pub struct Commit {
        pub hash: String,
        pub author: String,
        pub time: i64,
        pub summary: String,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Kind { Feature, Fix, Docs, Other }

    impl Commit {
        pub fn kind(&self) -> Kind
    }

kind() looks at the summary prefix: "feat" gives Feature, "fix" gives Fix,
"docs" gives Docs, anything else Other.

    pub fn parse_commit(line: &str) -> Option<Commit>

parses `hash|author|time|summary` into a Commit (the summary is everything
after the third `|`, and may itself contain `|`). It returns None if a field
is missing or the time is not an integer.

Keep every earlier function working. Test, fmt, clippy, commit."#,
    quiz: &[
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What does #[derive(PartialEq)] on a struct give you?",
            choices: &[
                "The == operator for the struct",
                "Printing with {:?}",
                "A .clone() method",
                "Public fields",
            ],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Which receiver lets a method change the struct's fields?",
            choices: &["&self", "&mut self", "Self", "no receiver"],
            answer: 1,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "This does not compile:\n\n    let c = Commit { hash: \"a1\".to_string(), time: 5 };\n\nwhere Commit has fields hash, author, time and summary. Pick the fix.",
            choices: &[
                "Give every field: add author and summary",
                "Add #[derive(Default)] and nothing else",
                "Write Commit::hash = ...",
                "Make the fields pub",
            ],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "How is an associated function like `fn new(hash: &str) -> Commit` (no self) called?",
            choices: &[
                "Commit::new(\"a1\")",
                "c.new(\"a1\")",
                "new Commit(\"a1\")",
                "Commit.new(\"a1\")",
            ],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "How does Rust represent a value that may be absent?",
            choices: &[
                "null",
                "Option<T> with Some(v) or None",
                "a zero value",
                "an exception",
            ],
            answer: 1,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What does `\"12x\".parse::<i64>().ok()` evaluate to?",
            choices: &["Some(12)", "None", "0", "It panics"],
            answer: 1,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "This fails with E0004 non-exhaustive patterns:\n\n    match kind {\n        Kind::Feature => 1,\n        Kind::Fix => 2,\n    }\n\n(Kind also has Docs and Other.) Pick the fix.",
            choices: &[
                "Add a `_ => 0` arm (or arms for Docs and Other)",
                "Use if instead of match",
                "Add #[derive(PartialEq)] to Kind",
                "Put the arms in alphabetical order",
            ],
            answer: 0,
            tags: &[O3, O2],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What does `if let Some(c) = opt { .. }` do when opt is None?",
            choices: &[
                "Panics",
                "Skips the block",
                "Runs the block with c = 0",
                "Does not compile",
            ],
            answer: 1,
            tags: &[O3],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "This does not compile:\n\n    let time: i64 = text.parse::<i64>().ok();\n\nPick the fix inside a function returning Option<Commit>.",
            choices: &[
                "let time: i64 = text.parse::<i64>().ok()?;",
                "let time: i64 = text.parse::<i64>();",
                "let time: Option = text.parse().ok();",
                "let time = text as i64;",
            ],
            answer: 0,
            tags: &[O3],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "In a match, what does the arm `s if s.starts_with(\"fix\") => Kind::Fix` use?",
            choices: &["A guard", "A closure", "A trait", "A loop label"],
            answer: 0,
            tags: &[O3],
        },
    ],
    checks: &[
        Check {
            kind: CheckKind::Test("m04_parse_commit_fields"),
            tags: &[O1, O3],
        },
        Check {
            kind: CheckKind::Test("m04_parse_commit_rejects_short_line"),
            tags: &[O2, O3],
        },
        Check {
            kind: CheckKind::Test("m04_kind_from_prefix"),
            tags: &[O1, O2],
        },
    ],
    hidden_tests: include_str!("../../../curriculum/hidden/m04.rs"),
};
