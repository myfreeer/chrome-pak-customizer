use std::env;

#[derive(Debug, PartialEq)]
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
    pub self_name: Option<String>,
    pub edge_v5: bool,
}

enum PakArgParseState {
    Init,
    Command,
    InputPath,
    OutputPath
}

const U8_SLASH: u8 = '/' as u8;
const U8_HYPHEN: u8 = '-' as u8;
const U8_H: u8 = 'h' as u8;
const U8_P: u8 = 'p' as u8;
const U8_U: u8 = 'u' as u8;
const U8_E: u8 = 'e' as u8;
const HELP: &str = "--help";

#[inline]
fn self_name() -> Option<String> {
    let path = env::current_exe().ok()?;
    let file_name = path.file_name()?.to_str()?;
    Some(String::from(file_name))
}

#[inline]
fn is_empty(opt: &Option<String>) -> bool {
    match opt {
        None => true,
        Some(str) => str.is_empty()
    }
}

pub fn parse_args() -> PakArgs {
    parse_args_iter(env::args())
}

fn parse_option_arg(arg: &str, args: &mut PakArgs) {
    let mut is_option = false;
    for b in arg.as_bytes() {
        if !is_option && (b == &U8_SLASH || b == &U8_HYPHEN) {
            is_option = true;
            continue;
        }
        if !is_option {
            break;
        }
        match b {
            &U8_H => {
                args.command = PakCommand::Help;
                return;
            }
            &U8_P => {
                if args.command == PakCommand::Unknown {
                    args.command = PakCommand::Pack;
                }
            }
            &U8_U => {
                if args.command == PakCommand::Unknown {
                    args.command = PakCommand::Unpack;
                }
            }
            &U8_E => {
                args.edge_v5 = true;
            }
            _ => {}
        }
    }
}

fn parse_args_iter<I>(iter: I) -> PakArgs
where
    I: IntoIterator<Item = String>,
{
    let mut args = PakArgs {
        command: PakCommand::Unknown,
        input_path: None,
        output_path: None,
        self_name: self_name(),
        edge_v5: false,
    };
    let mut state = PakArgParseState::Init;

    for x in iter {
        match state {
            PakArgParseState::Init => {
                if is_empty(&args.self_name) {
                    args.self_name = Some(x);
                }
                state = PakArgParseState::Command;
            }
            PakArgParseState::Command => {
                if x.eq_ignore_ascii_case(HELP) {
                    args.command = PakCommand::Help;
                }
                parse_option_arg(x.as_str(), &mut args);
                if args.command == PakCommand::Help {
                    return args;
                } else if args.command == PakCommand::Unknown {
                    continue;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> PakArgs {
        parse_args_iter(args.iter().map(|x| String::from(*x)).collect::<Vec<String>>())
    }

    #[test]
    fn compact_options_allow_edge_modifier_after_command() {
        let args = parse(&["pak", "-ue", "in.pak", "out"]);
        assert_eq!(args.command, PakCommand::Unpack);
        assert!(args.edge_v5);
        assert_eq!(args.input_path.as_deref(), Some("in.pak"));
        assert_eq!(args.output_path.as_deref(), Some("out"));
    }

    #[test]
    fn compact_options_allow_edge_modifier_before_command() {
        let args = parse(&["pak", "-eu", "in.pak", "out"]);
        assert_eq!(args.command, PakCommand::Unpack);
        assert!(args.edge_v5);
        assert_eq!(args.input_path.as_deref(), Some("in.pak"));
        assert_eq!(args.output_path.as_deref(), Some("out"));
    }
}
