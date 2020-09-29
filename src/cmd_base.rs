use clap::{App, ArgMatches};

pub trait CmdBase {
    const NAME: &'static str;
    fn subcommand<'a, 'b>() -> App<'a, 'b>;
    fn run(args: &ArgMatches) -> Result<(), String>;
}
