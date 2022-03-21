use chumsky::prelude::*;

#[derive(Debug)]
pub struct Proc {
    pub name: String,
    pub ops: Vec<Operation>,
}

#[derive(Debug, Clone)]
pub enum Operation {
    Integer(u64),

    // Arithmetic
    Add,
    Sub,
    Mul,
    DivMod,
    IDivMod,

    // Intrinsics
    Drop,
    Print,
}

pub fn parser() -> impl Parser<char, Vec<Proc>, Error = Simple<char>> {
    proc_parser().repeated().then_ignore(end())
}

pub fn proc_parser() -> impl Parser<char, Proc, Error = Simple<char>> {
    // todo: parse type signatures
    text::keyword("proc")
        .ignore_then(text::ident().padded())
        .then_ignore(text::keyword("in").padded())
        .then(op_parser())
        .then_ignore(text::keyword("end").padded())
        .map(|(name, ops)| Proc { name, ops })
}

pub fn op_parser() -> impl Parser<char, Vec<Operation>, Error = Simple<char>> {
    choice((
        just('+').to(Operation::Add),
        just('-').to(Operation::Sub),
        just('*').to(Operation::Mul),
        text::keyword("divmod").to(Operation::DivMod),
        text::keyword("idivmod").to(Operation::IDivMod),
        text::keyword("drop").to(Operation::Drop),
        text::keyword("print").to(Operation::Print),
        text::int(10).map(|s: String| Operation::Integer(s.parse().unwrap())),
    )).then_ignore(just(' ').repeated()).padded().repeated()
}
