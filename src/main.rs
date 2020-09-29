#[macro_use]
extern crate clap;

mod ami;
mod cmd;
mod cmd_base;
mod fs;
mod io;

use clap::App;
use cmd::configure::Configure;
use cmd::transcribe::Transcribe;
use cmd_base::CmdBase;

fn main() {
    let maches = App::new(crate_name!())
        .author(crate_authors!())
        .about(crate_description!())
        .version(crate_version!())
        .subcommand(Configure::subcommand())
        .subcommand(Transcribe::subcommand())
        .get_matches();

    let result = match maches.subcommand() {
        (Configure::NAME, Some(args)) => Configure::run(&args),
        (Transcribe::NAME, Some(args)) => Transcribe::run(&args),
        _ => Err("No subcommand chosen. Add --help | -h to view the subcommands.".to_string()),
    };
    if let Err(msg) = result {
        eprint!("{}", msg);
        std::process::exit(1);
    }
}
