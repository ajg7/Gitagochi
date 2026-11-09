use crate::academy::curriculum::{
    Check, CheckKind, Item, ItemKind, Module, ObjId, Objective, Section, Topic,
};

const O1: ObjId = ObjId(9, 1);
const O2: ObjId = ObjId(9, 2);
const O3: ObjId = ObjId(9, 3);

pub const MODULE: Module = Module {
    title: "Lifetimes",
    topic: Topic::Lifetimes,
    objectives: &[
        Objective {
            id: O1,
            text: "Annotate lifetimes on functions that return references",
            section: "Lifetime annotations",
        },
        Objective {
            id: O2,
            text: "Define structs that hold references",
            section: "Structs that borrow",
        },
        Objective {
            id: O3,
            text: "Apply the lifetime elision rules and know when annotations are needed",
            section: "Elision rules",
        },
    ],
    sections: &[
        Section {
            title: "Lifetime annotations",
            body: r#"Every reference is valid for some region of the program, its lifetime. Most
of the time the compiler infers it. When a function returns a reference
derived from more than one reference argument, you must say which input the
result borrows from, with a lifetime parameter:

    pub fn longer<'a>(a: &'a str, b: &'a str) -> &'a str {
        if b.len() > a.len() { b } else { a }
    }

`'a` is a name (any `'name`), declared in <..> like a type parameter. The
signature says: the result lives at most as long as both a and b. Without it
the compiler reports error[E0106] missing lifetime specifier ("this function's
return type contains a borrowed value, but the signature does not say whether
it is borrowed from `a` or `b`").

Annotations do not change how long anything lives; they describe the
relationship so the compiler can check callers. A caller cannot keep the
result after either input is dropped:

    let r;
    {
        let s = String::from("long one");
        r = longer("x", &s);
    }                          // s dropped here
    println!("{r}");           // error[E0597]: `s` does not live long enough

You can never return a reference to a local variable, whatever the
annotation; return an owned value instead."#,
        },
        Section {
            title: "Structs that borrow",
            body: r#"A struct may hold references, so it can view data without copying it. It
then needs a lifetime parameter, and every reference field uses it:

    pub struct Fields<'a> {
        pub hash: &'a str,
        pub author: &'a str,
    }

A `Fields<'a>` cannot outlive the text it borrows from: if the line is
dropped, the Fields value must already be gone. In return, building one
allocates nothing; each field is a slice of the original line.

Write the lifetime wherever the type is named: `Fields<'a>` in signatures,
or `Fields<'_>` to let the compiler fill in an elided one:

    pub fn split_two(line: &str) -> Option<Fields<'_>> {
        let (hash, author) = line.split_once('|')?;
        Some(Fields { hash, author })
    }

`s.split_once('|')` splits at the first `|` into Some((before, after)), or
None if there is none. Struct field init shorthand `Fields { hash, author }`
uses variables of the same names.

A struct with an owned String field needs no lifetime; prefer owned fields
unless avoiding the copy matters."#,
        },
        Section {
            title: "Elision rules",
            body: r#"Elision lets you leave lifetimes out where the answer is unambiguous. The
compiler applies three rules to a function signature:

1. Each reference parameter gets its own lifetime.
2. If there is exactly one input lifetime, the output gets it.
3. If one parameter is &self or &mut self, the output gets self's lifetime.

So these need no annotations:

    fn first_word(s: &str) -> &str                       // rule 2
    fn longest_summary(commits: &[Commit]) -> Option<&str>   // rule 2
    fn name(&self) -> &str                               // rule 3

while `fn longer(a: &str, b: &str) -> &str` has two inputs and no self, so
it needs explicit <'a>.

Clippy prefers elision: writing `fn f<'a>(s: &'a str) -> &'a str` where the
rules would give the same result triggers clippy::needless_lifetimes. Write
`fn f(s: &str) -> &str` instead, and `Fields<'_>` for a struct in the
return type.

A `&'static str` lives for the whole program; string literals have this
type."#,
        },
    ],
    read_more: &["https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html"],
    brief: r#"Step 9: zero-copy parsing. Add to src/parse.rs:

    pub struct Fields<'a> {
        pub hash: &'a str,
        pub author: &'a str,
        pub time: &'a str,
        pub summary: &'a str,
    }

    pub fn split_fields(line: &str) -> Option<Fields<'_>>

splits a line into its four fields, each a slice of `line` (the summary is
everything after the third `|`); None if a field is missing.

Add to src/stats.rs:

    pub fn longest_summary(commits: &[Commit]) -> Option<&str>

the summary with the most characters (the first one on a tie), borrowed from
the commits; None for no commits.

    pub fn longer<'a>(a: &'a str, b: &'a str) -> &'a str

the longer of two strings by characters, `a` on a tie.

Re-export all four from the crate root with pub use. Keep every earlier
function and path working. Test, fmt, clippy, commit."#,
    quiz: &[
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What does `fn longer<'a>(a: &'a str, b: &'a str) -> &'a str` promise?",
            choices: &[
                "The result is valid no longer than both a and b",
                "a and b live forever",
                "The result is an owned String",
                "a and b are copied",
            ],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "This fails with E0106 missing lifetime specifier:\n\n    fn pick(a: &str, b: &str) -> &str { a }\n\nPick the fix.",
            choices: &[
                "fn pick<'a>(a: &'a str, b: &str) -> &'a str",
                "fn pick(a: &str, b: &str) -> &'static mut str",
                "fn pick(a: str, b: str) -> str",
                "fn pick(a: &str, b: &str) -> &&str",
            ],
            answer: 0,
            tags: &[O1, O3],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Do lifetime annotations make a value live longer?",
            choices: &[
                "No; they only describe relationships the compiler checks",
                "Yes; 'a extends the value's life",
                "Only for Strings",
                "Only in release builds",
            ],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "This does not compile:\n\n    fn make() -> &str {\n        let s = String::from(\"hi\");\n        &s\n    }\n\nPick the fix.",
            choices: &[
                "Return the owned String: fn make() -> String { String::from(\"hi\") }",
                "Add <'a> and return &'a str",
                "Return &mut s",
                "Mark s as static mut",
            ],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Why does `struct Fields { hash: &str }` not compile?",
            choices: &[
                "A reference field needs a lifetime: struct Fields<'a> { hash: &'a str }",
                "Structs cannot hold references",
                "Fields must be pub",
                "&str must be String",
            ],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What is the benefit of `Fields<'a>` holding &'a str fields?",
            choices: &[
                "No copying: each field is a slice of the original line",
                "It can outlive the line",
                "It can be sent anywhere without limits",
                "It is always faster to compile",
            ],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What does `\"a|b|c\".split_once('|')` return?",
            choices: &[
                "Some((\"a\", \"b|c\"))",
                "Some((\"a\", \"b\"))",
                "[\"a\", \"b\", \"c\"]",
                "None",
            ],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Why does `fn longest_summary(commits: &[Commit]) -> Option<&str>` need no annotation?",
            choices: &[
                "There is one input lifetime, so the output gets it",
                "Option never needs lifetimes",
                "Commit is Copy",
                "It does need one",
            ],
            answer: 0,
            tags: &[O3],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "Clippy flags this with clippy::needless_lifetimes:\n\n    pub fn first<'a>(s: &'a str) -> &'a str { .. }\n\nPick the fix.",
            choices: &[
                "pub fn first(s: &str) -> &str",
                "pub fn first<'a, 'b>(s: &'a str) -> &'b str",
                "pub fn first(s: &'static str) -> &str",
                "pub fn first(s: String) -> &str",
            ],
            answer: 0,
            tags: &[O3],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "For a method `fn name(&self, other: &str) -> &str`, whose lifetime does the result get?",
            choices: &["self's", "other's", "'static", "It is an error"],
            answer: 0,
            tags: &[O3],
        },
    ],
    checks: &[
        Check {
            kind: CheckKind::Test("m09_split_fields_borrows"),
            tags: &[O2],
        },
        Check {
            kind: CheckKind::Test("m09_longest_summary"),
            tags: &[O3],
        },
        Check {
            kind: CheckKind::Test("m09_longest_summary_empty_none"),
            tags: &[O3],
        },
        Check {
            kind: CheckKind::Test("m09_longer_annotated"),
            tags: &[O1],
        },
    ],
    hidden_tests: include_str!("../../../curriculum/hidden/m09.rs"),
};
