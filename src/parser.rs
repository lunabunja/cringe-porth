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

    // Arithmetic
    Add,
    Sub,
    Mul,
    DivMod,
    IDivMod,

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

pub fn proc_parser<'i>()
    -> impl Parser<&'i str, Proc<'i>, Error = Simple<&'i str>>
{
    // todo: parse type signatures
    just("proc")
        .ignore_then(any())
        .then_ignore(just("in"))
        .then(op_parser())
        .then_ignore(just("end"))
        .map(|(name, ops)| Proc { name, ops })
}

pub fn const_parser<'i>()
    -> impl Parser<&'i str, Const<'i>, Error = Simple<&'i str>>
{
    just("const")
        .ignore_then(any())
        .then(op_parser())
        .then_ignore(just("end"))
        .map(|(name, ops)| Const { name, ops })
}

pub fn op_parser<'i>()
    -> impl Parser<&'i str, Vec<Operation<'i>>, Error = Simple<&'i str>>
{
    choice((
        just("+").to(Operation::Add),
        just("-").to(Operation::Sub),
        just("*").to(Operation::Mul),
        just("divmod").to(Operation::DivMod),
        just("idivmod").to(Operation::IDivMod),
        just("drop").to(Operation::Drop),
        just("dup").to(Operation::Dup),
        just("print").to(Operation::Print),
        just("swap").to(Operation::Swap),
        any().try_map(|s: &str, span| Ok(Operation::Integer(
            s.parse().map_err(|e| Simple::custom(span, format!("{}", e)))?
        ))),
        filter(|s| *s != "end").map(Operation::Word)
    )).recover_with(skip_then_retry_until(["end"])).repeated()
}
