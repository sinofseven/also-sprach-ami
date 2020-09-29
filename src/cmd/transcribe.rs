use crate::ami::{AmiWebSocketClient, OutputType};
use crate::cmd_base::CmdBase;
use crate::fs::load_api_key;
use crate::io::get_input;
use clap::{Arg, ArgMatches, SubCommand};

const ARG_AUDIO_FILE: &str = "audio_file";
const ARG_API_KEY: &str = "api_key";
const ARG_AUDIO_FORMAT: &str = "audio_format";
const ARG_NO_LOG: &str = "no_log";
const ARG_GRAMMAR_FILE_NAMES: &str = "grammar_file_names";
const ARG_OUTPUT_FILE: &str = "output_file";
const ARG_VERBOSE: &str = "verbose";
const ARG_TRACE: &str = "trace";
const ARG_OUTPUT_JSON: &str = "output_json";

pub struct Transcribe;

impl CmdBase for Transcribe {
    const NAME: &'static str = "transcribe";

    fn subcommand<'a, 'b>() -> clap::App<'a, 'b> {
        SubCommand::with_name(Self::NAME)
            .about("transcribe audio by ami voice cloud platform")
            .arg(
                Arg::with_name(ARG_AUDIO_FILE)
                    .long("audio-path")
                    .required(true)
                    .takes_value(true),
            )
            .arg(
                Arg::with_name(ARG_OUTPUT_FILE)
                    .long("output-file")
                    .required(true)
                    .takes_value(true)
            )
            .arg(
                Arg::with_name(ARG_API_KEY)
                    .long("api-key")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name(ARG_AUDIO_FORMAT)
                    .long("audio-foramt")
                    .takes_value(true)
                    .default_value("16k"),
            )
            .arg(
                Arg::with_name(ARG_GRAMMAR_FILE_NAMES)
                    .long("grammar-file-names")
                    .takes_value(true)
                    .default_value("-a-general"),
            )
            .arg(
                Arg::with_name(ARG_NO_LOG)
                    .long("no-log")
                    .takes_value(false)
                    .multiple(true),
            )
            .arg(
                Arg::with_name(ARG_VERBOSE)
                    .long("verbose")
                    .short("v")
                    .takes_value(false)
                    .multiple(true)
            )
            .arg(
                Arg::with_name(ARG_TRACE)
                    .long("trace")
                    .takes_value(false)
                    .multiple(true)
            )
            .arg(
                Arg::with_name(ARG_OUTPUT_JSON)
                    .long("is-json-output")
                    .takes_value(false)
                    .multiple(true)
            )
    }

    fn run(args: &ArgMatches) -> Result<(), String> {
        let api_key = if let Some(api_key) = args.value_of(ARG_API_KEY) {
            api_key.to_string()
        } else if let Some(api_key) = load_api_key()? {
            api_key
        } else {
            get_api_key()?
        };

        let audio_file_path = args.value_of(ARG_AUDIO_FILE).unwrap().to_string();
        let output_file_path = args.value_of(ARG_OUTPUT_FILE).unwrap();
        let audio_format = args.value_of(ARG_AUDIO_FORMAT).unwrap().to_string();
        let grammar_file_names = args.value_of(ARG_GRAMMAR_FILE_NAMES).unwrap().to_string();

        let is_no_log = args.occurrences_of(ARG_NO_LOG) > 0;
        let is_verbose = args.flag_of(ARG_VERBOSE);
        let is_trace = args.flag_of(ARG_TRACE);
        let is_output_json = args.flag_of(ARG_OUTPUT_JSON);

        let output_type = resolve_output_type(is_verbose, is_trace);

        let mut client = AmiWebSocketClient::new(
            api_key,
            audio_format,
            grammar_file_names,
            !is_no_log,
            is_output_json,
            audio_file_path,
            output_file_path,
            output_type,
        )?;

        client.exec()
    }
}

fn get_api_key() -> Result<String, String> {
    let mut s = String::new();
    while {
        s = get_input("AmiVoice Cloud Platform API KEY: ")?;

        s.is_empty()
    } {}

    Ok(s)
}

trait ArgMachesExt {
    fn flag_of(& self, name: &str) -> bool;
}

impl<'a> ArgMachesExt for ArgMatches<'a> {
    fn flag_of(&self, name: &str) -> bool {
        self.occurrences_of(name) > 0
    }
}

fn resolve_output_type(is_verbose: bool, is_trace: bool) -> OutputType {
    if is_trace {
        OutputType::Trace
    } else if is_verbose {
        OutputType::Verbose
    } else {
        OutputType::Nil
    }
}