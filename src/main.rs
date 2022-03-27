use std::path::Path;

use chumsky::{Parser, Stream};
use inkwell::{context::Context, targets::{Target, InitializationConfig, TargetMachine, RelocMode, CodeModel, FileType}, OptimizationLevel, passes::PassManager, module::Module};

use crate::codegen::Compiler;

mod codegen;
mod parser;

fn main() {
    let filename = std::env::args().nth(1).unwrap();
    let src = std::fs::read_to_string(&filename).unwrap();
    // fixme: this doesn't count whitespace so it's pretty much useless
    let mut last_token_pos = 0;
    let defs = parser::parser().parse(Stream::from_iter(
        src.len()..src.len() + 1,
        src.split_whitespace().map(|s| {
            last_token_pos += s.len();
            (s, last_token_pos - s.len()..last_token_pos)
        }),
    )).unwrap();

    Target::initialize_all(&InitializationConfig::default());
    // todo: enable all optimizations, change target based on arguments, etc.
    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).unwrap();
    let target_machine = target.create_target_machine(
        &triple,
        "generic",
        "",
        OptimizationLevel::Default,
        RelocMode::Default,
        CodeModel::Default).unwrap();

    let context = Context::create();
    let module = context.create_module(&filename);
    let builder = context.create_builder();

    let mut compiler = Compiler::new(&builder, &context, &module, &defs);
    compiler.module.set_data_layout(&target_machine.get_target_data().get_data_layout());
    compiler.module.set_triple(&triple);
    compiler.compile();

    let pass_manager: PassManager<Module> = PassManager::create(());
    // todo: passes here
    pass_manager.run_on(&module);

    compiler.module.print_to_file("a.ll").unwrap();
    target_machine.write_to_file(&module, FileType::Object, Path::new("a.out")).unwrap();
}
