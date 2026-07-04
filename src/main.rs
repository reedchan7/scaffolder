use clap::Parser;
use owo_colors::OwoColorize;

use scaffolder::cli::{Cli, Command};
use scaffolder::commands;

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::New(args) => commands::new::run(args),
        Command::List => commands::list::run(),
        Command::SelfUpdate => commands::self_update::run(),
        Command::Agent(args) => commands::agent::run(args),
    };
    if let Err(err) = result {
        eprintln!("{} {err:#}", "error:".red().bold());
        std::process::exit(1);
    }
}
