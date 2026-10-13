use std::process::ExitCode;

fn main() -> ExitCode {
    let args = match gitagochi::parse_args(std::env::args().skip(1)) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}\n\n{}", gitagochi::USAGE);
            return ExitCode::from(2);
        }
    };
    if args.help {
        println!("{}", gitagochi::USAGE);
        return ExitCode::SUCCESS;
    }
    match gitagochi::run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
