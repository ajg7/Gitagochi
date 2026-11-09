use crate::academy::curriculum::{
    Check, CheckKind, Item, ItemKind, Module, ObjId, Objective, Section, Topic,
};

const O1: ObjId = ObjId(6, 1);
const O2: ObjId = ObjId(6, 2);
const O3: ObjId = ObjId(6, 3);

pub const MODULE: Module = Module {
    title: "Collections and iterators",
    topic: Topic::Collections,
    objectives: &[
        Objective {
            id: O1,
            text: "Store data in Vec, HashMap and BTreeMap",
            section: "Collections",
        },
        Objective {
            id: O2,
            text: "Transform data with iterator adapters and collect",
            section: "Iterators",
        },
        Objective {
            id: O3,
            text: "Write closures and pass them to iterator and sort methods",
            section: "Closures",
        },
    ],
    sections: &[
        Section {
            title: "Collections",
            body: r#"`Vec<T>` is a growable list: `Vec::new()`, `vec![1, 2]`, `v.push(x)`,
`v.len()`, `v[i]` (panics if out of range), `v.get(i)` (an Option),
`v.sort()`, `v.dedup()` (removes neighbouring duplicates, so sort first).
`v.truncate(n)` keeps at most the first n elements.

A map stores values by key. `HashMap<K, V>` (std::collections) has no
defined order. `BTreeMap<K, V>` keeps its keys sorted, so iterating it gives
keys in ascending order; prefer it when output order matters.

    use std::collections::BTreeMap;
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    *counts.entry("ann".to_string()).or_insert(0) += 1;

`entry(key).or_insert(default)` returns a &mut to the value for key,
inserting the default first if the key is missing; `*` then updates it.
`map.get(&key)` returns Option<&V>; `map.len()` counts keys.

Iterating a map yields (key, value) pairs:

    for (name, n) in &counts { println!("{name}: {n}"); }"#,
        },
        Section {
            title: "Iterators",
            body: r#"An iterator produces items one at a time. `v.iter()` yields &T, `v.iter_mut()`
yields &mut T, `v.into_iter()` yields T (consuming v); `s.lines()` and
`map.iter()` are iterators too.

Adapters build a new iterator and do nothing until consumed:

    .map(f)          transform each item
    .filter(p)       keep items where p returns true
    .enumerate()     pair each item with its index
    .take(n)         stop after n items
    .rev()           reverse (for slices and ranges)

Consumers drive the iterator: `.count()`, `.sum()`, `.max()`, `.any(p)`,
`.find(p)`, and `.collect()`, which builds a collection whose type you name:

    let days: Vec<i64> = commits.iter().map(|c| c.time / 86_400).collect();
    let pairs: Vec<(String, usize)> = counts.into_iter().collect();

A BTreeMap or HashMap can also be collected from (key, value) pairs.
Calling .map(..) without a consumer computes nothing (and the compiler warns
"unused `Map` that must be used")."#,
        },
        Section {
            title: "Closures",
            body: r#"A closure is an inline anonymous function: `|args| body`.

    let double = |x: i64| x * 2;
    double(4)                         // 8
    v.iter().filter(|c| c.author == who)   // captures `who` from the scope

Closures can use variables from the surrounding scope; that is what makes
them useful as arguments to adapters.

Sorting with a custom order takes a closure that compares two items and
returns std::cmp::Ordering:

    pairs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

`x.cmp(&y)` compares in ascending order, so `b.cmp(&a)` is descending.
`.then(..)` breaks ties with a second comparison. Here: count descending,
then name ascending.

`sort_by_key(|p| p.1)` sorts by one key, ascending."#,
        },
    ],
    read_more: &[
        "https://doc.rust-lang.org/book/ch08-00-common-collections.html",
        "https://doc.rust-lang.org/book/ch13-01-closures.html",
        "https://doc.rust-lang.org/book/ch13-02-iterators.html",
    ],
    brief: r#"Step 6: per-author stats. Add to src/lib.rs (use std::collections::BTreeMap):

    pub fn commits_per_author(commits: &[Commit]) -> BTreeMap<String, usize>

how many commits each author made.

    pub fn top_authors(commits: &[Commit], n: usize) -> Vec<(String, usize)>

the n authors with the most commits as (author, count), most commits first;
authors with equal counts in name order. Fewer than n authors gives them all.

    pub fn streak(commits: &[Commit]) -> u32

the longest run of consecutive days with at least one commit (use day_of and
longest_streak). The commits may be in any order.

Print the top 3 authors from main. Keep every earlier function working.
Test, fmt, clippy, commit."#,
    quiz: &[
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Which map iterates its keys in sorted order?",
            choices: &["HashMap", "BTreeMap", "Both", "Neither"],
            answer: 1,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What does `*counts.entry(k).or_insert(0) += 1;` do?",
            choices: &[
                "Adds 1 to k's count, starting from 0 if k is new",
                "Inserts 1 for k only if k is new",
                "Panics if k is missing",
                "Resets k's count to 0",
            ],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "`v.dedup()` leaves duplicates in [3, 1, 3]. Pick the fix to remove all duplicates.",
            choices: &[
                "Call v.sort() before v.dedup()",
                "Call v.dedup() twice",
                "Use v.truncate(2)",
                "Use v.iter().dedup()",
            ],
            answer: 0,
            tags: &[O1],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What does `v.iter().map(|x| x * 2)` compute on its own?",
            choices: &[
                "Nothing until it is consumed, e.g. by collect or sum",
                "A Vec of doubled values",
                "It doubles v in place",
                "The sum of doubled values",
            ],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "This does not compile (type annotations needed):\n\n    let days = commits.iter().map(|c| c.time / 86_400).collect();\n\nPick the fix.",
            choices: &[
                "let days: Vec<i64> = ... .collect();",
                "Replace collect() with count()",
                "Add .iter() after collect()",
                "Use into_iter() instead of iter()",
            ],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "Which adapter keeps only the items for which a closure returns true?",
            choices: &["map", "filter", "take", "enumerate"],
            answer: 1,
            tags: &[O2, O3],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What does `.take(3)` do?",
            choices: &[
                "Stops after at most 3 items",
                "Skips the first 3 items",
                "Takes ownership of 3 items",
                "Panics with fewer than 3 items",
            ],
            answer: 0,
            tags: &[O2],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "In `commits.iter().filter(|c| c.author == who)`, where does `who` come from?",
            choices: &[
                "The closure captures it from the surrounding scope",
                "It is a global",
                "filter passes it in",
                "It must be a parameter of the closure",
            ],
            answer: 0,
            tags: &[O3],
        },
        Item {
            kind: ItemKind::SpotTheBug,
            prompt: "This should sort by count, highest first, but sorts lowest first:\n\n    pairs.sort_by(|a, b| a.1.cmp(&b.1));\n\nPick the fix.",
            choices: &[
                "pairs.sort_by(|a, b| b.1.cmp(&a.1));",
                "pairs.sort();",
                "pairs.sort_by_key(|p| p.1);",
                "pairs.dedup();",
            ],
            answer: 0,
            tags: &[O3],
        },
        Item {
            kind: ItemKind::MultipleChoice,
            prompt: "What does `.then(a.0.cmp(&b.0))` add to a comparison?",
            choices: &[
                "A tie-break used when the first comparison is equal",
                "A second sort pass over the whole list",
                "Reverses the order",
                "Nothing; it is ignored",
            ],
            answer: 0,
            tags: &[O3],
        },
    ],
    checks: &[
        Check {
            kind: CheckKind::Test("m06_counts_per_author"),
            tags: &[O1],
        },
        Check {
            kind: CheckKind::Test("m06_top_authors_sorted"),
            tags: &[O2, O3],
        },
        Check {
            kind: CheckKind::Test("m06_streak_from_commits"),
            tags: &[O1, O2],
        },
    ],
    hidden_tests: include_str!("../../../curriculum/hidden/m06.rs"),
};
