use crate::academy::curriculum::{
    Check, CheckKind, Item, ItemKind, Module, ObjId, Objective, Section, Topic,
};

const O1: ObjId = ObjId(5, 1);
const O2: ObjId = ObjId(5, 2);
const O3: ObjId = ObjId(5, 3);

pub const MODULE: Module = Module {
    title: "Error handling with Result and ?",
    topic: Topic::Errors,
    objectives: &[
        Objective {
            id: O1,
            text: "Return Result for operations that can fail and handle both outcomes",
            section: "Result",
        },
        Objective {
            id: O2,
            text: "Propagate errors with the ? operator",
            section: "The ? operator",
        },
        Objective {
            id: O3,
            text: "Define a custom error enum and keep panics out of library code",
            section: "Error types",
        },
    ],
    sections: &[
        Section {
            title: "Result",
            body: r#"Recoverable failures are values of `Result<T, E>`, an enum from std:

    enum Result<T, E> {
        Ok(T),    // success, holding the value
        Err(E),   // failure, holding the error
    }

`"42".parse::<i64>()` returns Result<i64, ParseIntError>. Handle it with
match, like any enum:

    match text.parse::<i64>() {
        Ok(n) => println!("{n}"),
        Err(e) => println!("not a number: {e}"),
    }

`.unwrap()` and `.expect("msg")` return the Ok value but panic on Err; use
them only where failure is a bug, never on input a user controls.
`.ok()` turns a Result into an Option, dropping the error.
`.map_err(|e| ...)` replaces the error value with another (the |e| ... part is
a small inline function; module 6 covers these).

A failing operation is a normal outcome, not an exception: the caller decides
what to do. A test can compare a whole Result, for example
`parse_line("x", 3) == Err(ParseError::MissingField(3))`, when both the
value and error types derive PartialEq."#,
        },
        Section {
            title: "The ? operator",
            body: r#"`expr?` on a Result means: if it is Ok(v), continue with v; if it is
Err(e), return Err(e) from the current function at once. The function must
itself return a Result with a compatible error type.

    fn parse_all(lines: &[&str]) -> Result<Vec<i64>, ParseIntError> {
        let mut out = Vec::new();
        for l in lines {
            out.push(l.parse::<i64>()?);   // the first bad line is returned
        }
        Ok(out)
    }

When the error types differ, convert first:

    let time = text.parse::<i64>().map_err(|_| ParseError::BadTime(n))?;

The final value of a function returning Result must be wrapped: `Ok(out)`,
not `out`.

To know each line's number while looping, `for (i, line) in log.lines().enumerate()`
gives the 0-based index i with each line, so the 1-based number is i + 1.

`?` works the same way on Option (returning None) in a function returning
Option. You cannot use `?` in a function returning plain i64 or (); that is
error[E0277]. `main` may return `std::process::ExitCode` instead:

    use std::process::ExitCode;
    fn main() -> ExitCode {
        if let Err(e) = run() {
            eprintln!("error: {e:?}");
            return ExitCode::FAILURE;   // a non-zero exit code
        }
        ExitCode::SUCCESS
    }

`eprintln!` prints to stderr, where error messages belong."#,
        },
        Section {
            title: "Error types",
            body: r#"A library should say what went wrong with a typed error, so callers can react
to each case. An enum with one variant per failure works well:

    #[derive(Debug, PartialEq)]
    pub enum ParseError {
        MissingField(usize),   // 1-based line number
        BadTime(usize),
    }

    pub fn parse_line(line: &str, n: usize) -> Result<Commit, ParseError> {
        ...
        if fields_missing {
            return Err(ParseError::MissingField(n));
        }
        ...
    }

Library code should not panic on bad input: no unwrap() on user data, no
indexing past the end, no panic!. Return an Err and let the binary decide
whether to print it and exit. Panics are for bugs (a broken invariant), not
for bad input files.

Carry the context the caller needs (here the line number) inside the variant.
Deriving Debug lets main print the error with {:?}."#,
        },
    ],
    read_more: &[
        "https://doc.rust-lang.org/book/ch09-00-error-handling.html",
        "https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html",
        "https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html",
    ],
    brief: r#"Step 5: report bad lines instead of skipping them. Add to src/lib.rs:

    #[derive(Debug, PartialEq)]
    pub enum ParseError {
        MissingField(usize),
        BadTime(usize),
    }

    pub fn parse_line(line: &str, n: usize) -> Result<Commit, ParseError>

parses one line like parse_commit; `n` is its 1-based line number. A line
with fewer than four fields gives Err(MissingField(n)); a time that is not an
integer gives Err(BadTime(n)).

    pub fn parse_log(log: &str) -> Result<Vec<Commit>, ParseError>

parses every non-blank line in order, returning the first error. Line numbers
count every line of the log, blank ones included.

Change src/main.rs to use parse_log: print the number of commits, or print the
error to stderr and exit with a non-zero code. No unwrap or expect on the log.

Keep every earlier function working. Test, fmt, clippy, commit."#,
    quiz: &[
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Which type should a function return when it can fail on bad input?",
            choices: &["Result<T, E>", "T, and panic on failure", "bool", "()"],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What does `.unwrap()` do on an Err?",
            choices: &["Returns a default", "Panics", "Returns None", "Retries"],
            answer: 1,
            tags: &[O1, O3],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What does `.ok()` do to a Result?",
            choices: &[
                "Turns it into an Option, dropping the error",
                "Panics on Err",
                "Returns true on success",
                "Converts the error type",
            ],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "In a function returning Result<Vec<i64>, E>, what does `x?` do when x is Err(e)?",
            choices: &[
                "Returns Err(e) from the function immediately",
                "Panics",
                "Skips the value and continues",
                "Replaces it with 0",
            ],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "This fails with error[E0277] (the ? operator can only be used in a function that returns Result or Option):\n\n    fn total(text: &str) -> i64 {\n        let n: i64 = text.parse()?;\n        n * 2\n    }\n\nPick the fix.",
            choices: &[
                "Return Result<i64, ParseIntError> and end with Ok(n * 2)",
                "Remove the type annotation on n",
                "Write text.parse()?? instead",
                "Add #[derive(Debug)]",
            ],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "This does not compile:\n\n    fn parse_all(..) -> Result<Vec<Commit>, ParseError> {\n        let mut out = Vec::new();\n        ...\n        out\n    }\n\nPick the fix.",
            choices: &[
                "End with Ok(out)",
                "End with out?",
                "End with Some(out)",
                "End with return out;",
            ],
            answer: 0,
            tags: &[O2, O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "How do you use `?` when the error is ParseIntError but the function returns Result<_, ParseError>?",
            choices: &[
                "text.parse::<i64>().map_err(|_| ParseError::BadTime(n))?",
                "text.parse::<i64>()? as ParseError",
                "You cannot; use unwrap",
                "text.parse::<ParseError>()?",
            ],
            answer: 0,
            tags: &[O2, O3],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Why give each failure its own enum variant, like MissingField(usize)?",
            choices: &[
                "Callers can match on each case and get its context",
                "It makes the program faster",
                "Rust forbids String errors",
                "So that the program panics with a message",
            ],
            answer: 0,
            tags: &[O3],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "A library function crashes the program on a bad log line:\n\n    let time: i64 = parts[2].parse().unwrap();\n\nPick the fix for library code.",
            choices: &[
                "Return Err(ParseError::BadTime(n)) instead, e.g. with map_err and ?",
                "Use expect(\"bad time\") instead",
                "Wrap the call in panic!",
                "Use parse::<u8>() instead",
            ],
            answer: 0,
            tags: &[O3],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Which return type lets main exit with a non-zero code without panicking?",
            choices: &[
                "std::process::ExitCode",
                "bool",
                "i32 only on Unix",
                "Option<()>",
            ],
            answer: 0,
            tags: &[O2],
        },
    ],
    checks: &[
        Check {
            kind: CheckKind::Test("m05_parse_log_ok"),
            tags: &[O1, O2],
        },
        Check {
            kind: CheckKind::Test("m05_reports_missing_field_line"),
            tags: &[O2, O3],
        },
        Check {
            kind: CheckKind::Test("m05_reports_bad_time_line"),
            tags: &[O1, O3],
        },
    ],
    hidden_tests: include_str!("../../../curriculum/hidden/m05.rs"),
};
