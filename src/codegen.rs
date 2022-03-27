use inkwell::{context::Context, module::Module, builder::Builder, values::{BasicValueEnum, BasicMetadataValueEnum, AggregateValueEnum}, types::{BasicMetadataTypeEnum, FunctionType, IntType}, IntPredicate};

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
            self.context.void_type().fn_type(
                &[BasicMetadataTypeEnum::IntType(self.context.i64_type())],
                false,
            ),
            None,
        );

        self.defs.iter().for_each(|def| {
            if let Definition::Proc(proc) = def {
                if proc.name == "main" {
                    self.compile_proc(proc);
                }
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

        function.get_param_iter().for_each(|arg| self.stack.push(arg));
        self.compile_ops(&proc.ops);

        if self.stack.is_empty() {
            self.builder.build_return(None);
        } else if self.stack.len() == 1 {
            self.builder.build_return(Some(&self.stack.pop().unwrap()));
        } else {
            let types = self.stack.iter()
                .map(|v| v.get_type()).collect::<Vec<_>>();
            let mut agg = AggregateValueEnum::StructValue(
                self.context.struct_type(&types, false).get_undef());
            self.stack.iter().enumerate().for_each(|(i, val)| {
                agg = self.builder.build_insert_value(
                    agg,
                    *val,
                    i.try_into().unwrap(),
                    "").unwrap();
            });
            self.builder.build_return(Some(&agg));
        }

        assert!(function.verify(true));
        self.stack = saved_stack;
    }

    pub fn compile_ops(&mut self, ops: &Vec<Operation>) {
        ops.iter().for_each(|op| {
            match op {
                Operation::Integer(i) => self.stack.push(self.context.i64_type().const_int(
                    *i, false
                ).into()),
                Operation::Word(word) => if let Some(proc) = self.module.get_function(word) {
                    // Procedure has been compiled before
                    let arguments = proc.get_param_iter().map(|_| {
                        self.stack.pop().unwrap().into()
                    }).collect::<Vec<BasicMetadataValueEnum<'ctx>>>();

                    if arguments.len() == 0 {
                        self.builder.build_call(proc, &[], "");
                    } else {
                        self.stack.push(
                            self.builder.build_call(proc, &arguments, "")
                                .try_as_basic_value().left().unwrap()
                        );
                    }
                } else {
                    self.defs.iter().for_each(|def| match def {
                        Definition::Const(constant)
                            => if constant.name == *word
                        {
                            self.compile_ops(&constant.ops);
                        },

                        Definition::Proc(proc) => if proc.name == *word {
                            // Procedure has not been compiled before
                            let caller = self.builder.get_insert_block().unwrap();
                            self.compile_proc(proc);

                            self.builder.position_at_end(caller);

                            let fn_value = self.module.get_function(word).unwrap();

                            let mut arguments = vec![];
                            fn_value.get_param_iter().for_each(|_|
                                arguments.push(self.stack.pop().unwrap().into()));
                            arguments.reverse();

                            if let Some(ret_value) = self.builder.build_call(
                                    fn_value,
                                    &arguments,
                                    ""
                                ).try_as_basic_value().left()
                            {
                                if let BasicValueEnum::StructValue(struct_value) = ret_value {
                                    // procedure returned multiple values
                                    struct_value.get_type()
                                        .get_field_types()
                                        .iter()
                                        .enumerate()
                                        .for_each(|(i, _)|
                                            self.stack.push(self.builder
                                                .build_extract_value(
                                                    struct_value,
                                                    i.try_into().unwrap(),
                                                    ""
                                                ).unwrap())
                                        );
                                } else {
                                    self.stack.push(ret_value);
                                }
                            }
                        },
                    });
                },

                Operation::If(then_branch) => {
                    let function = self.builder.get_insert_block().unwrap()
                        .get_parent().unwrap();

                    let then_block = self.context.append_basic_block(function, "then");
                    let else_block = self.context.append_basic_block(function, "else");
                    let cont_block = self.context.append_basic_block(function, "ifcont");

                    let cond = self.stack.pop().unwrap().into_int_value();
                    self.builder.build_conditional_branch(cond, then_block, else_block);

                    self.builder.position_at_end(then_block);
                    self.compile_ops(then_branch);
                    self.builder.build_unconditional_branch(cont_block);

                    self.builder.position_at_end(else_block);
                    // fixme: implement else branches
                    self.builder.build_unconditional_branch(cont_block);

                    self.builder.position_at_end(cont_block);
                },
    
                // Arithmetic
                Operation::Add => {
                    let y = self.stack.pop().unwrap().into_int_value();
                    let x = self.stack.pop().unwrap().into_int_value();
    
                    self.stack.push(self.builder.build_int_add(x, y, "tmpadd").into());
                },
                Operation::Sub => {
                    let y = self.stack.pop().unwrap().into_int_value();
                    let x = self.stack.pop().unwrap().into_int_value();
    
                    self.stack.push(self.builder.build_int_sub(x, y, "tmpsub").into());
                },
                Operation::Mul => {
                    let y = self.stack.pop().unwrap().into_int_value();
                    let x = self.stack.pop().unwrap().into_int_value();
    
                    self.stack.push(self.builder.build_int_mul(x, y, "tmpmul").into());
                },
                Operation::DivMod => {
                    let y = self.stack.pop().unwrap().into_int_value();
                    let x = self.stack.pop().unwrap().into_int_value();
    
                    self.stack.push(self.builder.build_int_unsigned_div(x, y, "tmpdiv").into());
                    self.stack.push(self.builder.build_int_unsigned_rem(x, y, "tmpmod").into());
                },
                Operation::IDivMod => {
                    let y = self.stack.pop().unwrap().into_int_value();
                    let x = self.stack.pop().unwrap().into_int_value();
    
                    self.stack.push(self.builder.build_int_signed_div(x, y, "tmpidiv").into());
                    self.stack.push(self.builder.build_int_signed_rem(x, y, "tmpimod").into());
                },
                Operation::Equal => {
                    let y = self.stack.pop().unwrap().into_int_value();
                    let x = self.stack.pop().unwrap().into_int_value();

                    self.stack.push(self.builder.build_int_compare(
                        IntPredicate::EQ,
                        x,
                        y,
                        "tmpeq").into());
                },

                // Intrinsics
                Operation::Drop => {
                    self.stack.pop();
                },
                Operation::Dup => {
                    self.stack.push(self.stack.last().unwrap().clone());
                },
                Operation::Print => {
                    let x = self.stack.pop().unwrap();

                    self.builder.build_call(
                        self.module.get_function("print").unwrap(),
                        &[x.into()],
                        "print"
                    );
                },
                Operation::Swap => {
                    let x = self.stack.pop().unwrap().into_int_value();
                    let y = self.stack.pop().unwrap().into_int_value();

                    self.stack.push(x.into());
                    self.stack.push(y.into());
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
    if out_types.is_empty() {
        context.void_type().fn_type(&convert_types(context, in_types), false)
    } else if out_types.len() == 1 {
        type_to_fn_type(context, in_types, &out_types[0])
    } else {
        context.struct_type(&convert_types(context, out_types), false)
            .fn_type(&convert_types(context, in_types), false)
    }
}

fn type_to_fn_type<'ctx>(
    context: &'ctx Context,
    in_types: &Vec<Type>,
    out_type: &Type) -> FunctionType<'ctx>
{
    match out_type {
        Type::Int =>
            context.i64_type().fn_type(&convert_types(context, in_types), false),
    }
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
