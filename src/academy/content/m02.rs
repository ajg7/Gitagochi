use crate::academy::curriculum::{
    Check, CheckKind, Item, ItemKind, Module, ObjId, Objective, Section, Topic,
};

const O1: ObjId = ObjId(2, 1);
const O2: ObjId = ObjId(2, 2);
const O3: ObjId = ObjId(2, 3);

pub const MODULE: Module = Module {
    title: "Types, variables and control flow",
    topic: Topic::Basics,
    objectives: &[
        Objective {
            id: O1,
            text: "Use integer types and bind values with let, mut and shadowing",
            section: "Variables and integer types",
        },
        Objective {
            id: O2,
            text: "Branch and repeat with if, loop, while and for",
            section: "Control flow",
        },
        Objective {
            id: O3,
            text: "Write functions whose bodies are expressions",
            section: "Functions and expressions",
        },
    ],
    sections: &[
        Section {
            title: "Variables and integer types",
            body: r#"`let` binds a value to a name. Bindings are immutable unless marked `mut`:

    let x = 5;
    x = 6;          // error[E0384]: cannot assign twice to immutable variable
    let mut y = 5;
    y += 1;         // fine

Shadowing declares a new variable with the same name; it may even change type:

    let n = "42";
    let n: i64 = n.parse().unwrap_or(0); // a new `n`, now an i64

Integer types state their size and sign: i8 i16 i32 i64 i128 (signed) and
u8 u16 u32 u64 u128 (unsigned). `usize` is the unsigned size of an index or
length (what `.len()` returns). Without a hint, an integer literal is i32.
Underscores make literals readable: 86_400.

Rust never converts between integer types silently; use `as`:

    let len: usize = 3;
    let n = len as u32;

Integer division truncates: 7 / 2 == 3, and 86_399 / 86_400 == 0.
In a debug build, arithmetic that overflows panics ("attempt to add with
overflow"); u32 cannot hold a negative number.

`const` defines a constant; it needs a type and an UPPER_CASE name:

    const SECONDS_PER_DAY: i64 = 86_400;"#,
        },
        Section {
            title: "Control flow",
            body: r#"`if` takes a bool condition (no parentheses needed, and integers are not
treated as bools). `if` is an expression, so both branches must have the same
type:

    let label = if n > 0 { "some" } else { "none" };

`loop` repeats until `break`; `break value` makes the loop produce a value:

    let mut i = 0;
    let first_big = loop {
        i += 1;
        if i * i > 50 { break i; }
    };

`while cond { .. }` repeats while the condition holds.

`for` walks anything iterable: ranges and slices.

    for i in 0..3 { }        // 0, 1, 2 (end excluded)
    for i in 0..=3 { }       // 0, 1, 2, 3
    for d in days { }        // each element of a slice `days: &[i64]`
    for pair in days.windows(2) {
        // pair[0] and pair[1] are neighbours: (a, b), then (b, c), ...
    }

`continue` skips to the next iteration. Prefer `for` over manual index loops:
it cannot go out of bounds. A slice of n elements has n - 1 windows of 2, and
none when it is empty or holds one element."#,
        },
        Section {
            title: "Functions and expressions",
            body: r#"Functions declare the type of every parameter and of the result:

    fn add(a: i64, b: i64) -> i64 {
        a + b
    }

A block's value is its last expression, written without a semicolon. Adding a
semicolon turns it into a statement whose value is `()`, so this fails with
error[E0308] mismatched types (expected `i64`, found `()`):

    fn add(a: i64, b: i64) -> i64 {
        a + b;
    }

`return value;` leaves a function early; use it for early exits, not for the
last line:

    fn longest(days: &[i64]) -> u32 {
        if days.is_empty() {
            return 0;
        }
        // ...
        1
    }

Blocks, `if` and `match` are all expressions, so they can be assigned or
returned directly. Methods such as `a.max(b)` return the larger of two numbers
of the same type."#,
        },
    ],
    read_more: &[
        "https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html",
        "https://doc.rust-lang.org/book/ch03-02-data-types.html",
        "https://doc.rust-lang.org/book/ch03-03-how-functions-work.html",
        "https://doc.rust-lang.org/book/ch03-05-control-flow.html",
    ],
    brief: r#"Step 2: streaks. Commit timestamps are unix seconds. Add to src/lib.rs:

    pub fn day_of(ts: i64) -> i64

the day number of timestamp `ts` (ts >= 0): whole days since 1970-01-01, so
0..=86_399 is day 0 and 86_400 is day 1.

    pub fn longest_streak(days: &[i64]) -> u32

the length of the longest run of consecutive days in `days`. The input is
sorted ascending and may repeat a day (several commits on one day count once).
An empty slice gives 0. Example: [3, 4, 4, 5, 9, 10] gives 3.

Keep count_lines working. Add unit tests, run fmt and clippy, and commit."#,
    quiz: &[
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Which type does `.len()` on a slice return?",
            choices: &["i32", "u32", "usize", "i64"],
            answer: 2,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "This fails with error[E0384]:\n\n    let count = 0;\n    count += 1;\n\nPick the fix.",
            choices: &[
                "let mut count = 0;",
                "const count = 0;",
                "let count: u32 = 0;",
                "count = count + 1;",
            ],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What is 86_399 / 86_400 for i64 values?",
            choices: &["1", "0", "0.99998", "a compile error"],
            answer: 1,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "After `let n = \"7\"; let n: i64 = n.parse().unwrap_or(0);` what is `n`?",
            choices: &[
                "A compile error: n is immutable",
                "The i64 7, a new variable shadowing the old one",
                "The &str \"7\"",
                "A mutable reference",
            ],
            answer: 1,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Which values does `for i in 0..3` produce?",
            choices: &["0, 1, 2", "0, 1, 2, 3", "1, 2, 3", "3 only"],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "How many items does `days.windows(2)` yield for a slice of 1 element?",
            choices: &["0", "1", "2", "It panics"],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What does `let x = loop { break 5; };` set x to?",
            choices: &["()", "5", "true", "It does not compile"],
            answer: 1,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "This fails with mismatched types (expected `i64`, found `()`):\n\n    fn double(x: i64) -> i64 {\n        x * 2;\n    }\n\nPick the fix.",
            choices: &[
                "Remove the semicolon: x * 2",
                "Change the return type to ()",
                "Write let x * 2;",
                "Add mut to x",
            ],
            answer: 0,
            tags: &[O3],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "This does not compile:\n\n    let label = if n > 0 { \"some\" } else { 0 };\n\nPick the fix.",
            choices: &[
                "Make both branches the same type, e.g. else { \"none\" }",
                "Wrap the condition in parentheses",
                "Remove the else branch",
                "Use while instead of if",
            ],
            answer: 0,
            tags: &[O2, O3],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Where is `return` the idiomatic choice?",
            choices: &[
                "On the last line of every function",
                "For an early exit, such as an empty input",
                "Inside every if branch",
                "Never; Rust has no return",
            ],
            answer: 1,
            tags: &[O3],
        },
    ],
    checks: &[
        Check {
            kind: CheckKind::Test("m02_day_of_rounds_down"),
            tags: &[O1, O3],
        },
        Check {
            kind: CheckKind::Test("m02_streak_counts_consecutive_days"),
            tags: &[O2],
        },
        Check {
            kind: CheckKind::Test("m02_streak_ignores_repeats"),
            tags: &[O2],
        },
        Check {
            kind: CheckKind::Test("m02_streak_empty_is_zero"),
            tags: &[O2, O3],
        },
    ],
    hidden_tests: include_str!("../../../curriculum/hidden/m02.rs"),
};
