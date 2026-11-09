use crate::academy::curriculum::{
    Check, CheckKind, Item, ItemKind, Module, ObjId, Objective, Section, Topic,
};

const O1: ObjId = ObjId(3, 1);
const O2: ObjId = ObjId(3, 2);
const O3: ObjId = ObjId(3, 3);

pub const MODULE: Module = Module {
    title: "Ownership and borrowing",
    topic: Topic::Ownership,
    objectives: &[
        Objective {
            id: O1,
            text: "Explain moves, Copy types versus String, and when to clone",
            section: "Ownership and moves",
        },
        Objective {
            id: O2,
            text: "Pass data by & and &mut reference following the borrow rules",
            section: "References and borrowing",
        },
        Objective {
            id: O3,
            text: "Return and use &str slices that borrow from a String or &str",
            section: "String slices",
        },
    ],
    sections: &[
        Section {
            title: "Ownership and moves",
            body: r#"Every value has exactly one owner. When the owner goes out of scope the value
is dropped (its memory freed). Assigning or passing a heap value such as a
String moves it: the old name can no longer be used.

    let a = String::from("ann");
    let b = a;              // a is moved into b
    println!("{a}");        // error[E0382]: borrow of moved value: `a`

Small fixed-size types (integers, bool, char, f64, and &str references) are
Copy: assignment copies them and the original stays usable.

    let x = 5;
    let y = x;              // x is still usable

To keep using the original of a non-Copy value, clone it (a deep copy that
costs an allocation):

    let b = a.clone();      // a and b are both usable

Passing a String to a function by value moves it into the function. A
function that only needs to read should take a reference instead (next
section).

A Vec<String> owns its Strings. `Vec::new()` creates one, `v.push(x)` moves x
into it, and `v.contains(&x)` checks for an equal element. To store text
borrowed from elsewhere, make an owned String with `s.to_string()`."#,
        },
        Section {
            title: "References and borrowing",
            body: r#"A reference borrows a value without taking ownership.

    &T      shared reference: read only; any number may exist at once
    &mut T  exclusive reference: may modify; only one, and no & at the same time

    fn total(xs: &[i64]) -> i64 { ... }        // reads
    fn bump(xs: &mut [i64]) {                    // modifies in place
        for x in xs.iter_mut() {
            *x += 1;                             // * reaches the value behind &mut
        }
    }

    let mut v = vec![1, 2];
    bump(&mut v);           // v is now [2, 3]

The borrow rules, checked at compile time: while a &mut exists, nothing else
may use the value; and a reference may not outlive the value it points to.

    let mut v = vec![1];
    let first = &v[0];
    v.push(2);              // error[E0502]: cannot borrow `v` as mutable
    println!("{first}");    //   because it is also borrowed as immutable

Prefer slice parameters: take &[T] rather than &Vec<T>, and &mut [T] rather
than &mut Vec<T> when you only change elements; clippy::ptr_arg flags the Vec
forms. A &Vec<T> or &mut Vec<T> argument is accepted where a slice is expected.
Take &mut Vec<T> only when the function must push or remove elements.

`s.to_lowercase()` returns a new lowercase String; assign it through the
reference to change the element: `*name = name.to_lowercase();`."#,
        },
        Section {
            title: "String slices",
            body: r#"`String` owns growable UTF-8 text. `&str` is a borrowed view (a slice) of
text owned by someone else: a String, or a literal baked into the program.

    let owned: String = String::from("a1|ann|100|fix");
    let view: &str = &owned;          // borrows the whole String
    let part: &str = &owned[0..2];    // "a1" (byte range)

Take &str parameters: they accept both `&String` and literals.

Splitting borrows pieces of the original, without copying:

    let line = "a1|ann|100|fix";
    let mut parts = line.split('|');  // an iterator of &str pieces
    parts.next();                     // Some("a1")

`line.split('|').nth(i)` gives the i-th piece (0-based) wrapped in an Option:
Some(piece), or None if there are fewer pieces. `.unwrap_or("")` turns that
into the piece or a default:

    let author: &str = line.split('|').nth(1).unwrap_or("");

A function that returns a slice of its &str argument returns `&str`, and the
compiler ties the result to the argument:

    pub fn first_word(s: &str) -> &str {
        s.split(' ').next().unwrap_or("")
    }

The result borrows from `s`, so it cannot outlive it. Returning a slice of a
String created inside the function is an error, because that String is dropped
when the function returns; return an owned String in that case."#,
        },
    ],
    read_more: &[
        "https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html",
        "https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html",
        "https://doc.rust-lang.org/book/ch04-03-slices.html",
    ],
    brief: r#"Step 3: fields and authors. Add to src/lib.rs:

    pub fn field(line: &str, i: usize) -> &str

the i-th `|`-separated field of a log line (0 = hash, 1 = author, 2 = time,
3 = summary), borrowed from `line`; "" if the line has fewer fields.

    pub fn authors(log: &str) -> Vec<String>

the distinct authors of the non-blank lines of `log`, in the order each first
appears.

    pub fn normalize(names: &mut [String])

lowercases every name in place (so "Ann@X.io" and "ann@x.io" match).

Keep everything from steps 1 and 2 working. Test, fmt, clippy, commit."#,
    quiz: &[
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "After `let a = String::from(\"x\"); let b = a;`, what happens if you use `a`?",
            choices: &[
                "It prints \"x\"",
                "A compile error: a was moved into b",
                "A runtime panic",
                "It is an empty String",
            ],
            answer: 1,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Which of these types is Copy?",
            choices: &["String", "Vec<i64>", "i64", "Vec<String>"],
            answer: 2,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "This fails with E0382 borrow of moved value:\n\n    let name = String::from(\"ann\");\n    let mut v = Vec::new();\n    v.push(name);\n    println!(\"{name}\");\n\nPick the fix that keeps both usable.",
            choices: &[
                "v.push(name.clone());",
                "v.push(&mut name);",
                "let name = &String::from(\"ann\"); v.push(name); with v: Vec<String>",
                "Move println! above let name",
            ],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "How many &mut references to one value may exist at the same time?",
            choices: &[
                "Any number",
                "One, and no & alongside it",
                "Two",
                "None; Rust has no &mut",
            ],
            answer: 1,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "Clippy flags this with clippy::ptr_arg:\n\n    pub fn total(xs: &Vec<i64>) -> i64 { .. }\n\nPick the fix.",
            choices: &[
                "pub fn total(xs: &[i64]) -> i64",
                "pub fn total(xs: Vec<i64>) -> i64",
                "pub fn total(xs: &mut Vec<i64>) -> i64",
                "pub fn total(xs: &&Vec<i64>) -> i64",
            ],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Inside `for name in names.iter_mut()`, how do you replace the element with its lowercase form?",
            choices: &[
                "name = name.to_lowercase();",
                "*name = name.to_lowercase();",
                "&name = name.to_lowercase();",
                "names.push(name.to_lowercase());",
            ],
            answer: 1,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "This fails with E0502:\n\n    let first = &v[0];\n    v.push(9);\n    println!(\"{first}\");\n\nPick the fix.",
            choices: &[
                "Print first before calling v.push(9)",
                "Make first a &mut",
                "Write v.push(&9)",
                "Declare v without mut",
            ],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What does `\"a|b\".split('|').nth(5).unwrap_or(\"\")` return?",
            choices: &["\"b\"", "\"\"", "It panics", "None"],
            answer: 1,
            tags: &[O3],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Why take `s: &str` instead of `s: &String` as a parameter?",
            choices: &[
                "&str accepts both &String and string literals",
                "&str is mutable",
                "&String cannot be read",
                "&str copies the text, which is faster",
            ],
            answer: 0,
            tags: &[O3],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "`pub fn first_word(s: &str) -> &str` returns a piece of `s`. What does the result borrow from?",
            choices: &[
                "A new String made by the function",
                "The argument s, so it cannot outlive s",
                "Nothing; &str is owned",
                "A global buffer",
            ],
            answer: 1,
            tags: &[O3],
        },
    ],
    checks: &[
        Check {
            kind: CheckKind::Test("m03_field_returns_slice"),
            tags: &[O3],
        },
        Check {
            kind: CheckKind::Test("m03_field_missing_is_empty"),
            tags: &[O3],
        },
        Check {
            kind: CheckKind::Test("m03_authors_unique_in_order"),
            tags: &[O1, O3],
        },
        Check {
            kind: CheckKind::Test("m03_normalize_lowercases_in_place"),
            tags: &[O2],
        },
    ],
    hidden_tests: include_str!("../../../curriculum/hidden/m03.rs"),
};
