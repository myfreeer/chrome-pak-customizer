use crate::pak_args::{PakArgs, PakCommand, parse_args};
use crate::pak_index::PakIndex;
use crate::pak_pack::pak_pack_index_path;
use crate::pak_unpack::pak_unpack_path;

mod pak_def;
mod pak_error;
mod pak_header;
mod pak_file;
mod pak_file_type;
mod pak_file_io;
mod pak_index;
mod pak_unpack;
mod pak_pack;
mod pak_args;
mod pak_brotli;

fn print_help(args: &PakArgs) {
    let default_name = String::from("pak");
    let self_name = args.self_name.as_ref().unwrap_or(&default_name);
    match args.command {
        PakCommand::Unknown => println!("Unknown command"),
        PakCommand::Help => {}
        PakCommand::Pack => println!("Incomplete pack arguments"),
        PakCommand::Unpack => println!("Incomplete unpack arguments")
    }
    println!(include_str!("pak_help.txt"), self_name);
}

fn main() {
    let args = parse_args();
    // TODO: output status
    // TODO: allow warnings
    match args.command {
        PakCommand::Unknown | PakCommand::Help => print_help(&args),
        PakCommand::Pack => {
            if args.input_path.is_none() || args.output_path.is_none() {
                print_help(&args);
                return;
            }
            match pak_pack_index_path(
                args.input_path.unwrap(),
                args.output_path.unwrap()) {
                Ok(_) => {}
                Err(err) => {
                    println!("Error packing: {:?}", err);
                }
            }
        }
        PakCommand::Unpack => {
            if args.input_path.is_none() || args.output_path.is_none() {
                print_help(&args);
                return;
            }
            match pak_unpack_path(
                args.input_path.unwrap(),
                args.output_path.unwrap()) {
                Ok(_) => {}
                Err(err) => {
                    println!("Error unpacking: {:?}", err);
                }
            }
        }
    }
}
