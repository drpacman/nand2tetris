mod compiler;
mod xmlcompiler;
mod vmcompiler;

use compiler::{ CompilationEngine };

fn main() {
    let filestem = "nand2tetris/projects/11/ComplexArrays/Main";
    let mut tokenizer = compiler::JackTokenizer::new(format!("{}.jack", filestem).as_str());
    let mut tokens = tokenizer.parse().unwrap();
    let mut compiler = xmlcompiler::XMLCompilationEngine::new(tokens.into_iter().peekable());
    let xml = compiler.compile_class();

    tokens = tokenizer.parse().unwrap();
    let mut vmcompiler = vmcompiler::VMCompiler::new(tokens.into_iter().peekable());
    let vm_instructions = vmcompiler.compile_class();
    let writer = vmcompiler::VMWriter::new( filestem );
    writer.write(&vm_instructions);    
}
