use assembler::{Parser, Assembler, AssemblyWriter};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filestem = args[1].as_str();
    let parser = Parser::new(filestem);
    let instructions = parser.parse().unwrap();
    let mut assembler = Assembler::new();
    let assembly = assembler.assemble(&instructions);
    let assembly_writer = AssemblyWriter::new(filestem);
    assembly_writer.write(assembly);
}

