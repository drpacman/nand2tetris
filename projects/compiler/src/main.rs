use compiler::{JackTokenizer, XMLCompilationEngine};


fn main() {
    let filename = "nand2tetris/projects/10/ArrayTest/Main.jack";
    let mut tokenizer = JackTokenizer::new(filename);
    let tokens = tokenizer.parse().unwrap();
    let mut compiler = XMLCompilationEngine::new(tokens.into_iter().peekable());
    compiler.compile_class();
}
