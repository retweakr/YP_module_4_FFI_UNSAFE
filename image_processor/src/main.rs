use clap::Parser;
use image_processor::{run, CliArgs};
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = CliArgs::parse();

    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
