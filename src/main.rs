use chumsky::Parser;
use inkwell::context::Context;

use crate::codegen::Compiler;

mod codegen;
mod parser;

fn main() {
    let filename = std::env::args().nth(1).unwrap();
    let src = std::fs::read_to_string(&filename).unwrap();
    let procs = parser::parser().parse(src).unwrap();

    let context = Context::create();
    let module = context.create_module(&filename);
    let builder = context.create_builder();

    let mut compiler = Compiler::new(&builder, &context, &module, &procs);
    compiler.compile();

    compiler.module.print_to_stderr();
}
