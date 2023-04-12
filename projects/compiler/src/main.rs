mod compiler;
mod xmlcompiler;

use 
compiler::CompilationEngine;

fn main() {
    let filename = "nand2tetris/projects/10/Square/Main.jack";
    let mut tokenizer = compiler::JackTokenizer::new(filename);
    let tokens = tokenizer.parse().unwrap();
    let mut compiler = xmlcompiler::XMLCompilationEngine::new(tokens.into_iter().peekable());
    compiler.compile_class();
}
