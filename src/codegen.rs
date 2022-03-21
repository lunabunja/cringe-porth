use inkwell::{context::Context, module::Module, builder::Builder, values::BasicValueEnum, types::BasicMetadataTypeEnum};

use crate::parser::{Operation, Proc};

#[derive(Debug)]
pub struct Compiler<'a, 'ctx> {
    pub module: &'a Module<'ctx>,
    builder: &'a Builder<'ctx>,
    context: &'ctx Context,
    procs: &'a Vec<Proc>,
    stack: Vec<BasicValueEnum<'ctx>>,
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    pub fn new(builder: &'a Builder<'ctx>, context: &'ctx Context, module: &'a Module<'ctx>, procs: &'a Vec<Proc>) -> Compiler<'a, 'ctx> {
        Compiler {
            module,
            builder,
            context,
            procs,
            stack: Vec::new(),
        }
    }

    pub fn compile(&mut self) {
        self.module.add_function(
            "print",
            self.context.i64_type().fn_type(
                &[BasicMetadataTypeEnum::IntType(self.context.i64_type())],
                false,
            ),
            None,
        );
        self.procs.iter().for_each(|proc| self.compile_proc(proc));
    }

    pub fn compile_proc(&mut self, proc: &Proc) {
        let saved_stack = self.stack.clone();
        self.stack.clear();

        let function = self.module.add_function(
            &proc.name,
            self.context.i64_type().fn_type(&[], false),
            None
        );

        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);

        proc.ops.iter().for_each(|op| {
            match *op {
                Operation::Integer(i) => {
                    self.stack.push(BasicValueEnum::IntValue(
                        self.context.i64_type().const_int(
                            i, false
                        )
                    ))
                },
    
                // Arithmetic
                Operation::Add => {
                    let y = self.stack.pop().unwrap().into_int_value();
                    let x = self.stack.pop().unwrap().into_int_value();
    
                    self.stack.push(BasicValueEnum::IntValue(self.builder.build_int_add(x, y, "tmpadd")));
                },
                Operation::Sub => {
                    let y = self.stack.pop().unwrap().into_int_value();
                    let x = self.stack.pop().unwrap().into_int_value();
    
                    self.stack.push(BasicValueEnum::IntValue(self.builder.build_int_sub(x, y, "tmpsub")));
                },
                Operation::Mul => {
                    let y = self.stack.pop().unwrap().into_int_value();
                    let x = self.stack.pop().unwrap().into_int_value();
    
                    self.stack.push(BasicValueEnum::IntValue(self.builder.build_int_mul(x, y, "tmpmul")));
                },
                Operation::DivMod => {
                    let y = self.stack.pop().unwrap().into_int_value();
                    let x = self.stack.pop().unwrap().into_int_value();
    
                    self.stack.push(BasicValueEnum::IntValue(self.builder.build_int_unsigned_div(x, y, "tmpdiv")));
                    self.stack.push(BasicValueEnum::IntValue(self.builder.build_int_unsigned_rem(x, y, "tmpmod")));
                },
                Operation::IDivMod => {
                    let y = self.stack.pop().unwrap().into_int_value();
                    let x = self.stack.pop().unwrap().into_int_value();
    
                    self.stack.push(BasicValueEnum::IntValue(self.builder.build_int_signed_div(x, y, "tmpidiv")));
                    self.stack.push(BasicValueEnum::IntValue(self.builder.build_int_signed_rem(x, y, "tmpimod")));
                },

                // Intrinsics
                Operation::Drop => {
                    self.stack.pop();
                },
                Operation::Print => {
                    let x = self.stack.pop().unwrap().into_int_value();
                    unimplemented!();
                }
            }
        });

        self.builder.build_return(Some(&self.stack.pop().unwrap().into_int_value()));

        assert!(function.verify(true));
        if !self.stack.is_empty() {
            todo!("Procedures are hardcoded to return a single u64 value");
        }

        self.stack = saved_stack;
    }
}
