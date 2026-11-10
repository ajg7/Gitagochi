use std::fs;

fn main() {
    let log = fs::read_to_string("log.txt").unwrap_or_default();
    println!("{} commits", tracker::count_lines(&log));
}
