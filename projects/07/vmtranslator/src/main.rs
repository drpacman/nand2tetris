use std::env;
// use std::io::Write;
use assembler::Instruction;
use vmtranslator::{Compiler, VMInstructionParser, ASMWriter};

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let filestem = args[1].as_str();
    // let filestem = "projects/07/MemoryAccess/PointerTest/PointerTest";
    let parser = VMInstructionParser::new(filestem);
    let vm_instructions = parser.parse().unwrap();
    let mut compiler = Compiler::new();
    let instructions : Vec<Instruction> = compiler.compile(vm_instructions);
    let writer = ASMWriter::new(filestem);
    writer.write(instructions);
}