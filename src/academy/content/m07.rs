use crate::academy::curriculum::{
    Check, CheckKind, Item, ItemKind, Module, ObjId, Objective, Section, Topic,
};

const O1: ObjId = ObjId(7, 1);
const O2: ObjId = ObjId(7, 2);
const O3: ObjId = ObjId(7, 3);

pub const MODULE: Module = Module {
    title: "Traits and generics",
    topic: Topic::Traits,
    objectives: &[
        Objective {
            id: O1,
            text: "Define and implement traits and use trait objects (dyn)",
            section: "Traits and trait objects",
        },
        Objective {
            id: O2,
            text: "Write generic functions with trait bounds",
            section: "Generics",
        },
        Objective {
            id: O3,
            text: "Implement std traits such as Display and use derives",
            section: "Standard traits",
        },
    ],
    sections: &[
        Section {
            title: "Traits and trait objects",
            body: r#"A trait names a set of methods that types can implement, like an interface:

    pub trait Stat {
        fn name(&self) -> String;
        fn value(&self, commits: &[Commit]) -> String;
    }

    pub struct TotalCommits;          // a unit struct: no fields

    impl Stat for TotalCommits {
        fn name(&self) -> String { "total commits".to_string() }
        fn value(&self, commits: &[Commit]) -> String { commits.len().to_string() }
    }

An impl must provide every method of the trait (error[E0046] otherwise).
Methods in a trait are public wherever the trait is; do not write `pub` on
them inside the impl. To call a trait method, the trait must be in scope
(`use tracker::Stat;`).

A trait object `&dyn Stat` is a reference to some value of any type that
implements Stat; the method to call is chosen at run time. That lets one
slice hold different types:

    let stats: [&dyn Stat; 2] = [&TotalCommits, &LongestStreak];
    for s in &stats { println!("{}: {}", s.name(), s.value(&commits)); }

A function can take `stats: &[&dyn Stat]` and accept a mix of types.
`n.to_string()` turns a number into a String; `format!("{a}: {b}")` builds
one from parts."#,
        },
        Section {
            title: "Generics",
            body: r#"A generic function works for many types. Type parameters go in <..>, and
trait bounds say what the type must support:

    pub fn largest<T: PartialOrd + Clone>(items: &[T]) -> Option<T> {
        let mut best = items.first()?.clone();
        for x in items {
            if *x > best { best = x.clone(); }
        }
        Some(best)
    }

Without the bound, the body may not compare or clone a T (error[E0369]
binary operation `>` cannot be applied to type `T`, or E0599 no method
`clone`). The bound `T: Ord` allows .cmp and use as a BTreeMap key; `Clone`
allows .clone(); `Ord` includes the comparison operators.

The compiler generates a copy of the function for each concrete type used
(monomorphization), so generics cost nothing at run time.

    largest(&[3, 9, 2])          // Some(9), T = i32
    largest(&["b", "a"])         // Some("b"), T = &str

A `where` clause moves long bounds out of the signature:

    fn f<T>(x: &[T]) -> usize where T: Ord + Clone { .. }"#,
        },
        Section {
            title: "Standard traits",
            body: r#"`Display` controls how `{}` prints a value (Debug controls `{:?}`). Implement
it with a `fmt` method that writes into the formatter:

    use std::fmt;

    impl fmt::Display for Commit {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{} {}: {}", self.hash, self.author, self.summary)
        }
    }

    println!("{c}");            // a1 ann@x.io: fix crash
    let s = c.to_string();      // every Display type gets to_string()

Display cannot be derived: you decide the text. Many other std traits can be
derived with #[derive(..)]: Debug, Clone, Copy (only if all fields are Copy),
PartialEq, Eq, PartialOrd, Ord, Hash, Default. Derived PartialOrd/Ord compare
fields in declaration order (for enums, variant order)."#,
        },
    ],
    read_more: &[
        "https://doc.rust-lang.org/book/ch10-01-syntax.html",
        "https://doc.rust-lang.org/book/ch10-02-traits.html",
        "https://doc.rust-lang.org/book/ch18-02-trait-objects.html",
    ],
    brief: r#"Step 7: pluggable stats. Add to src/lib.rs:

    pub trait Stat {
        fn name(&self) -> String;
        fn value(&self, commits: &[Commit]) -> String;
    }

    pub struct TotalCommits;   // name "total commits", value: number of commits
    pub struct LongestStreak;  // name "longest streak", value: streak(commits)

    pub fn report(stats: &[&dyn Stat], commits: &[Commit]) -> Vec<String>

one line "name: value" per stat, in order (e.g. "total commits: 3").

    impl std::fmt::Display for Commit

prints "hash author: summary", e.g. "a1 ann@x.io: fix crash".

    pub fn most_common<T: Ord + Clone>(items: &[T]) -> Option<T>

the value that occurs most often; on a tie, the smallest of the tied values;
None for an empty slice.

Use report in main. Keep every earlier function working. Test, fmt, clippy,
commit."#,
    quiz: &[
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What is a trait?",
            choices: &[
                "A named set of methods that types can implement",
                "A struct with no fields",
                "A kind of enum",
                "A macro",
            ],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What can a `&[&dyn Stat]` slice hold?",
            choices: &[
                "References to values of different types that all implement Stat",
                "Only values of one concrete type",
                "Owned Stat values",
                "Only unit structs",
            ],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "This fails with E0046 (missing `value` in implementation):\n\n    impl Stat for TotalCommits {\n        fn name(&self) -> String { \"total commits\".to_string() }\n    }\n\nPick the fix.",
            choices: &[
                "Also implement fn value(&self, commits: &[Commit]) -> String",
                "Mark name as pub fn",
                "Add #[derive(Stat)]",
                "Remove the trait method value",
            ],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "This fails with E0369 (binary operation `>` cannot be applied to type `T`):\n\n    fn biggest<T>(a: T, b: T) -> T { if a > b { a } else { b } }\n\nPick the fix.",
            choices: &[
                "fn biggest<T: PartialOrd>(a: T, b: T) -> T",
                "fn biggest<T: Display>(a: T, b: T) -> T",
                "fn biggest(a: dyn T, b: dyn T) -> T",
                "fn biggest<T>(a: &T, b: &T) -> T",
            ],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Which bound lets a generic T be used as a BTreeMap key and compared with .cmp?",
            choices: &["Ord", "Display", "Debug", "Default"],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What does the compiler do with a generic function used with i32 and &str?",
            choices: &[
                "Generates a separate copy for each type, with no run-time cost",
                "Boxes every value",
                "Checks types at run time",
                "Refuses: one function, one type",
            ],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Which trait controls how `{}` formats a value?",
            choices: &["Display", "Debug", "ToString only", "Clone"],
            answer: 0,
            tags: &[O3],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "This does not compile:\n\n    #[derive(Display)]\n    pub struct Commit { .. }\n\nPick the fix.",
            choices: &[
                "Write impl fmt::Display for Commit with a fmt method",
                "Use #[derive(Debug)] and print with {}",
                "Add pub to every field",
                "Write #[derive(ToString)]",
            ],
            answer: 0,
            tags: &[O3],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Once a type implements Display, which method does it get for free?",
            choices: &["to_string()", "clone()", "len()", "cmp()"],
            answer: 0,
            tags: &[O3],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "When can a struct derive Copy?",
            choices: &[
                "Only when all its fields are Copy",
                "Always",
                "Only when it has no methods",
                "Only when it implements Display",
            ],
            answer: 0,
            tags: &[O3],
        },
    ],
    checks: &[
        Check {
            kind: CheckKind::Test("m07_display_commit"),
            tags: &[O3],
        },
        Check {
            kind: CheckKind::Test("m07_stats_report"),
            tags: &[O1],
        },
        Check {
            kind: CheckKind::Test("m07_most_common_generic"),
            tags: &[O2],
        },
    ],
    hidden_tests: include_str!("../../../curriculum/hidden/m07.rs"),
};
