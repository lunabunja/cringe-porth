# cringe-porth
Alternative Porth compiler written in Rust. In this readme, porth refers to
[porth.porth](https://gitlab.com/tsoding/porth) while Porth refers to the Porth
language.

## Goals
- Most porth code should be compatible with cringe-porth. However, compliance
isn't a priority and correctly handling all edge cases isn't really required.
- Bootstrap porth without a prior version of porth.

## Non-Goals
- Replacing porth. This compiler is not intended to replace porth. At this
stage, this is just a toy for me to play with.
- Compliance. Do not try to make the compiler 100% compliant. Valid and
idiomatic Porth code should be correctly compiled, but covering all edge cases
is not a priority.
- Performance. Although I might do a few optimizations here and there, and I
will happily accept PRs that focus on performance, at this stage, optimization
isn't a priority.
- Error handling. Assume the input is already valid Porth code. The goal is to
bootstrap porth, which will be valid Porth code anyway.
- Extending the language. Do not add new features to Porth, if it is believed
that the feature is worth implementing, first shoot an issue to porth that
explains the feature, optionally shoot a PR that implements it, and if porth
accepts the feature, then it can be implemented in cringe-porth.
