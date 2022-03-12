use type_layout::TypeLayout;

use crate::pak_def::PakHeaderV5;

mod pak_def;
mod pak_error;

fn main() {
    println!("Hello, world!");
    println!("PakHeaderV5: {}", PakHeaderV5::type_layout())
}
