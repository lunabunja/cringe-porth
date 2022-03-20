use chumsky::Parser;
use inkwell::context::Context;

use crate::codegen::Compiler;

mod codegen;
mod parser;

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let procs = parser::parser().parse(src).unwrap();

    let context = Context::create();
    let module = context.create_module("main");
    let builder = context.create_builder();

    let mut compiler = Compiler::new(&builder, &context, &module);
    
    procs.iter().for_each(|proc| compiler.compile_proc(proc));

    compiler.module.print_to_stderr();
}
