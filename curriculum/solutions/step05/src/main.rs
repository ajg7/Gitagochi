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
            println!("{} commits", commits.len());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e:?}");
            ExitCode::FAILURE
        }
    }
}
