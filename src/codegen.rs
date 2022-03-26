use inkwell::{context::Context, module::Module, builder::Builder, values::{BasicValueEnum, BasicMetadataValueEnum}, types::{BasicMetadataTypeEnum, FunctionType, IntType}};

use crate::parser::{Operation, Proc, Definition, Type};

#[derive(Debug)]
pub struct Compiler<'a, 'ctx> {
    pub module: &'a Module<'ctx>,
    builder: &'a Builder<'ctx>,
    context: &'ctx Context,
    defs: &'a Vec<Definition<'a>>,
    stack: Vec<BasicValueEnum<'ctx>>,
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    pub fn new(
        builder: &'a Builder<'ctx>,
        context: &'ctx Context,
        module: &'a Module<'ctx>,
        defs: &'a Vec<Definition>) -> Compiler<'a, 'ctx>
    {
        Compiler {
            module,
            builder,
            context,
            defs,
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

        self.defs.iter().for_each(|def| {
            if let Definition::Proc(proc) = def {
                self.compile_proc(proc);
            }
        });
    }

    pub fn compile_proc(&mut self, proc: &Proc) {
        let saved_stack = self.stack.clone();
        self.stack.clear();

        let function = self.module.add_function(
            &proc.name,
            types_to_fn_type(self.context, &proc.inputs, &proc.outputs),
            None
        );

        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);

        self.compile_ops(&proc.ops);

        self.builder.build_return(Some(
            &self.context.const_struct(&self.stack.clone(), false)
        ));

        assert!(function.verify(true));

        self.stack = saved_stack;
    }

    pub fn compile_ops(&mut self, ops: &Vec<Operation>) {
        ops.iter().for_each(|op| {
            match *op {
                Operation::Integer(i) => {
                    self.stack.push(BasicValueEnum::IntValue(
                        self.context.i64_type().const_int(
                            i, false
                        )
                    ))
                },
                Operation::Word(word) => {
                    if let Some(proc) = self.module.get_function(word) {
                        self.stack.push(
                            // fixme: this doesn't handle void procs
                            self.builder.build_call(proc, &[], "")
                                .try_as_basic_value().left().unwrap()
                        );
                    } else {
                        if let Some(ops) = self.defs.iter()
                            .find_map(|def| match def {
                                Definition::Const(constant) => {
                                    if constant.name != word {
                                        None
                                    } else {
                                        Some(&constant.ops)
                                    }
                                },
                                _ => None,
                            }
                        ) {
                            self.compile_ops(ops);
                        } else {
                            panic!("Unknown word: {}", word);
                        }
                    }
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
                Operation::Dup => {
                    self.stack.push(self.stack.last().unwrap().clone());
                },
                Operation::Print => {
                    let x = self.stack.pop().unwrap().into_int_value();

                    self.builder.build_call(
                        self.module.get_function("print").unwrap(),
                        &[BasicMetadataValueEnum::IntValue(x)],
                        "print"
                    );
                },
                Operation::Swap => {
                    let x = self.stack.pop().unwrap().into_int_value();
                    let y = self.stack.pop().unwrap().into_int_value();

                    self.stack.push(BasicValueEnum::IntValue(x));
                    self.stack.push(BasicValueEnum::IntValue(y));
                },
            }
        });
    }
}

fn types_to_fn_type<'ctx>(
    context: &'ctx Context,
    in_types: &Vec<Type>,
    out_types: &Vec<Type>) -> FunctionType<'ctx>
{
    context.struct_type(&convert_types(context, out_types), false)
        .fn_type(&convert_types(context, in_types), false)
}

fn convert_types<'ctx, T: From<IntType<'ctx>>>(
    context: &'ctx Context,
    types: &Vec<Type>) -> Vec<T>
{
    types.iter().map(|t| {
        match t {
            Type::Int => context.i64_type().into(),
        }
    }).collect()
}
