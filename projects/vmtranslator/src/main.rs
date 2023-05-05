use std::env;
use std::path::{Path};
// use std::io::Write;
use assembler::{ Assembler, AssemblyWriter, Instruction };
use vmtranslator::{Compiler, VMInstructionParser, ASMWriter};

fn write_asm(filestem : &str, instructions : &Vec<Instruction>) {
    let writer = ASMWriter::new(filestem);
    writer.write(instructions);
}
fn write_hack(filestem : &str, instructions : &Vec<Instruction>) {
    let mut assembler = Assembler::new();
    let assembly = assembler.assemble(&instructions);
    let assembly_writer = AssemblyWriter::new(filestem);
    assembly_writer.write(assembly);
}

fn generate_asm(compiler: &mut Compiler, filename : &str) -> Vec<Instruction> {
    let parser = VMInstructionParser::new(filename);
    let vm_instructions = parser.parse().unwrap();
    compiler.compile(vm_instructions)    
}

fn compile(path : &Path, target : &str) {
    let mut compiler = Compiler::new();

    let dir_entries = std::fs::read_dir(path).expect(format!("File {:?} not found", path).as_str());    
    let vm_files : Vec<Result<std::fs::DirEntry, std::io::Error>> = dir_entries.into_iter().filter(|f| {
        f.as_ref().unwrap().path().as_path().extension().unwrap() == "vm"
    }).collect();

    let vm_file_count = vm_files.len();
    let mut compiled_instructions = vm_files.into_iter().map(|vm_file| {
        generate_asm(&mut compiler, vm_file.unwrap().path().to_str().unwrap())
    }).flatten().collect();
    // if more then one vm file exists, generate bootstrap code.
    let mut instructions = Vec::new();
    if vm_file_count > 1 {
        instructions.append(&mut compiler.generate_bootstrap());
    }
    instructions.append(&mut compiled_instructions);
    instructions.append(&mut compiler.generate_global_helper_functions());
    let mut target_file_stem = path.to_path_buf();
    target_file_stem.push(target);               
    write_asm(target_file_stem.to_str().unwrap(), &instructions);
    write_hack(target_file_stem.to_str().unwrap(), &instructions);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut path = Path::new(args[1].as_str());
    let target = args[2].as_str();
    if path.is_file(){
        path = path.parent().unwrap();
    }
    compile(path, target);
}