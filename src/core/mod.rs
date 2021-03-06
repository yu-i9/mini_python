#![feature(arbitrary_self_types)]
#![feature(iterator_find_map)]
pub mod token;
pub mod lexer;
pub mod syntax;
pub mod opcode;
pub mod compile;
pub mod parser;
pub mod env;
pub mod object;
pub mod eval;
pub mod error;
pub mod builtinmodule;
pub mod utils;
