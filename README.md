# cringe-porth
Alternative [Porth](https://gitlab.com/tsoding/porth) compiler written in Rust.

# Goals
- Most porth code should be compatible with cringe-porth. However, compliance
isn't a priority and correctly handling all edge cases isn't really required.
- Bootstrap `porth.porth` without a prior version of `porth.porth`.

# Non-Goals
- Replacing `porth.porth`. This compiler is not intended to replace
`porth.porth`. At this stage, this is just a toy for me to play with.
- Compliance. Do not try to make the compiler 100% compliant.
- Performance. Although I might do a few optimizations here and there, and I
will happily accept PRs that focus on performance, at this stage, optimization
isn't a priority.
- Error handling. Assume the input is already valid Porth code. The goal is to
bootstrap `porth.porth`, which will be valid Porth code anyway.
