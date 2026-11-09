use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let log = match fs::read_to_string("log.txt") {
        Ok(text) => text,
        Err(e) => {
            eprintln!("cannot read log.txt: {e}");
            return ExitCode::FAILURE;
        }
    };
    match tracker::parse_log(&log) {
        Ok(commits) => {
            let stats: [&dyn tracker::Stat; 2] = [&tracker::TotalCommits, &tracker::LongestStreak];
            for line in tracker::report(&stats, &commits) {
                println!("{line}");
            }
            for (author, n) in tracker::top_authors(&commits, 3) {
                println!("{author}: {n}");
            }
            let mut pet = tracker::pet::Pet::default();
            pet.feed(commits.len());
            println!("your pet is {}", pet.mood());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
