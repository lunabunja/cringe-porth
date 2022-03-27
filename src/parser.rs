use chumsky::prelude::*;

#[derive(Debug)]
pub enum Definition<'a> {
    Proc(Proc<'a>),
    Const(Const<'a>),
}

#[derive(Debug)]
pub struct Proc<'a> {
    pub name: &'a str,
    pub ops: Vec<Operation<'a>>,
    pub inputs: Vec<Type>,
    pub outputs: Vec<Type>,
}

#[derive(Debug, Clone)]
pub enum Type {
    Int,
}

#[derive(Debug)]
pub struct Const<'a> {
    pub name: &'a str,
    pub ops: Vec<Operation<'a>>,
}

#[derive(Debug, Clone)]
pub enum Operation<'a> {
    Integer(u64),
    Word(&'a str),

    If(Vec<Operation<'a>>),

    // Arithmetic
    Add, Sub, Mul, DivMod, IDivMod,
    Equal,

    // Intrinsics
    Drop,
    Dup,
    Print,
    Swap,
}

pub fn parser<'i>()
    -> impl Parser<&'i str, Vec<Definition<'i>>, Error = Simple<&'i str>>
{
    choice((
        proc_parser().map(Definition::Proc),
        const_parser().map(Definition::Const),
    )).repeated()
}

fn proc_parser<'i>()
    -> impl Parser<&'i str, Proc<'i>, Error = Simple<&'i str>>
{
    just("proc")
        .ignore_then(any())
        .then(type_parser().repeated())
        .then(just("--")
            .ignore_then(type_parser().repeated().at_least(1))
            .or_not())
        .then_ignore(just("in"))
        .then(op_parser())
        .then_ignore(just("end"))
        .map(|(((name, inputs), outputs), ops)| Proc {
            name, ops, inputs, outputs: outputs.unwrap_or_default()
        })
}

fn type_parser<'i>() -> impl Parser<&'i str, Type, Error = Simple<&'i str>> {
    just("int").to(Type::Int)
}

fn const_parser<'i>()
    -> impl Parser<&'i str, Const<'i>, Error = Simple<&'i str>>
{
    just("const")
        .ignore_then(any())
        .then(op_parser())
        .then_ignore(just("end"))
        .map(|(name, ops)| Const { name, ops })
}

fn op_parser<'i>()
    -> impl Parser<&'i str, Vec<Operation<'i>>, Error = Simple<&'i str>>
{
    recursive(|op_parser| {
        choice((
            just("+").to(Operation::Add),
            just("-").to(Operation::Sub),
            just("*").to(Operation::Mul),
            just("divmod").to(Operation::DivMod),
            just("idivmod").to(Operation::IDivMod),
            just("=").to(Operation::Equal),
            just("drop").to(Operation::Drop),
            just("dup").to(Operation::Dup),
            just("print").to(Operation::Print),
            just("swap").to(Operation::Swap),
            just("if").ignore_then(op_parser).then_ignore(just("end"))
                .map(|ops| Operation::If(ops)),
            any().try_map(|s: &str, span| Ok(Operation::Integer(
                s.parse().map_err(|e| Simple::custom(span, format!("{}", e)))?
            ))),
            filter(|s| *s != "end").map(Operation::Word)
        )).recover_with(skip_then_retry_until(["end"])).repeated()
    })
}
