use std::process::ExitCode;

fn main() -> ExitCode {
    match cmonitor_rs::run(std::env::args_os()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error:#}");
            ExitCode::FAILURE
        }
    }
}
