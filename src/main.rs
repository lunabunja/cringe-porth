use chumsky::{Parser, Stream};
use inkwell::context::Context;

use crate::codegen::Compiler;

mod codegen;
mod parser;

fn main() {
    let filename = std::env::args().nth(1).unwrap();
    let src = std::fs::read_to_string(&filename).unwrap();
    // fixme: this doesn't count whitespace so it's pretty much useless
    let mut last_token_pos = 0;
    let procs = parser::parser().parse(Stream::from_iter(
        src.len()..src.len() + 1,
        src.split_whitespace().map(|s| {
            last_token_pos += s.len();
            (s, last_token_pos - s.len()..last_token_pos)
        }),
    )).unwrap();

    eprintln!("{:?}", procs);

    let context = Context::create();
    let module = context.create_module(&filename);
    let builder = context.create_builder();

    let mut compiler = Compiler::new(&builder, &context, &module, &procs);
    compiler.compile();

    compiler.module.print_to_stderr();
}
