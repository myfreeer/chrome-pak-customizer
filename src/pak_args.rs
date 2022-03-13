use std::env;

#[derive(PartialEq)]
pub enum PakCommand {
    Unknown,
    Help,
    Pack,
    Unpack,
}

pub struct PakArgs {
    pub command: PakCommand,
    pub input_path: Option<String>,
    pub output_path: Option<String>,
    pub self_name: Option<String>
}

enum PakArgParseState {
    Init,
    Command,
    InputPath,
    OutputPath
}

const U8_H: u8 = 'h' as u8;
const U8_P: u8 = 'p' as u8;
const U8_U: u8 = 'u' as u8;
const HELP: &str = "--help";

#[inline]
fn self_name() -> Option<String> {
    match env::current_exe() {
        Ok(path) => Some(path.to_string_lossy().to_string()),
        Err(_) => None
    }
}

pub fn parse_args() -> PakArgs {
    let mut args = PakArgs {
        command: PakCommand::Unknown,
        input_path: None,
        output_path: None,
        self_name: self_name()
    };
    let mut state = PakArgParseState::Init;

    for x in env::args() {
        match state {
            PakArgParseState::Init => {
                if args.self_name.is_none() {
                    args.self_name = Some(x);
                }
                state = PakArgParseState::Command;
            }
            PakArgParseState::Command => {
                if x.eq_ignore_ascii_case(HELP) {
                    args.command = PakCommand::Help;
                }
                for b in x.as_bytes() {
                    match b {
                        &U8_H => args.command = PakCommand::Help,
                        &U8_P => args.command = PakCommand::Pack,
                        &U8_U => args.command = PakCommand::Unpack,
                        _ => {}
                    }
                }
                if args.command == PakCommand::Help {
                    return args;
                } else {
                    state = PakArgParseState::InputPath;
                }
            }
            PakArgParseState::InputPath => {
                args.input_path = Some(x);
                state = PakArgParseState::OutputPath;
            }
            PakArgParseState::OutputPath => {
                args.output_path = Some(x);
                break;
            }
        }
    }

    args
}