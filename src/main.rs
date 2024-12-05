use std::{
    io,
    num::ParseIntError,
    sync::mpsc::{self},
    time::Duration,
};

use clap::Parser;
use report::spawn_report_thread;
use ui::run_main;

mod external;
mod report;
mod ui;

#[derive(Parser)]
pub struct Command {
    #[clap(
        short,
        long,
        default_value = "1000",
        value_parser = parse_duration,
        help = "Interval in milliseconds"
    )]
    pub interval: Duration,
}

// tpo

fn parse_duration(arg: &str) -> Result<Duration, ParseIntError> {
    Ok(std::time::Duration::from_millis(arg.parse()?))
}

fn main() -> io::Result<()> {
    let cmd = Command::parse();

    let (tx, rx) = mpsc::channel();
    spawn_report_thread(tx, cmd.interval);
    run_main(rx)
}
