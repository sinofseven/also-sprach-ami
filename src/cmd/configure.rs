use crate::cmd_base::CmdBase;
use crate::fs::save_api_key;
use crate::io::get_input;
use clap::{ArgMatches, SubCommand};

pub struct Configure;

impl CmdBase for Configure {
    const NAME: &'static str = "configure";

    fn subcommand<'a, 'b>() -> clap::App<'a, 'b> {
        SubCommand::with_name(Self::NAME).about("configure AmiVoice Cloud Platform API KEY")
    }

    fn run(_: &ArgMatches) -> Result<(), String> {
        let api_key = get_api_key()?;
        save_api_key(&api_key)
    }
}

fn get_api_key() -> Result<String, String> {
    let mut s = String::new();
    while s.is_empty() {
        s = get_input("AmiVoice Cloud Platform API KEY: ")?;
    }

    Ok(s.clone())
}
