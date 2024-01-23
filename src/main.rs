//! Crate implementing an LLVM frontend for brainf*ck.

/// Module containing structures and logic for parsing brainf*ck code.
pub mod parser;

/// Module containing structures and logic for generating LLVM IR from parsed brainf*ck code.
pub mod codegen;

fn main() {
    let mut args = std::env::args();
    let executable = args.next().unwrap();

    match args.next() {
        Some(filename) => {
            let code = std::fs::read_to_string(filename).unwrap();
            let instructions = parser::parse(1, &code).unwrap();
            let ir = codegen::compile(&instructions).unwrap().to_string();

            println!("{ir}");
        },
        None => {
            println!("Usage: {executable} <filename>");
        }
    }
}
