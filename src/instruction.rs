use itertools::Itertools;
//use lazy_static::lazy_static;
use std::convert::TryFrom;

#[derive(Debug)]
pub enum InstructionArgType {
    Device,
    Register,
    Parameter,
    Number,
    Token,
}

impl TryFrom<&str> for InstructionArgType {
    type Error = &'static str;
    fn try_from(token: &str) -> Result<Self, Self::Error> {
        let arg = match token {
            "d" => Self::Device,
            "r" => Self::Register,
            "p" => Self::Parameter,
            "n" => Self::Number,
            "t" => Self::Token,
            _ => return Err("Invalid argument token"),
        };
        Ok(arg)
    }
}

pub enum InstructionArg<'a> {
    Device(&'a str),
    Register(&'a str),
    Number(u8),
}

//#[derive(Debug)]
pub struct Instruction<'a> {
    pub name: &'a str,
    pub f: fn(&[InstructionArg]),
    pub args: Vec<Vec<InstructionArgType>>,
}

impl<'a> Instruction<'a> {
    pub fn new(
        name: &'a str,
        f: fn(&[InstructionArg]),
        args: Vec<Vec<&str>>,
    ) -> Result<Self, &'static str> {
        let args = args
            .iter()
            .map(|types| {
                types
                    .iter()
                    .cloned()
                    .map(InstructionArgType::try_from)
                    .try_collect()
            })
            .try_collect()?;
        Ok(Self { name, f, args })
    }
}

macro_rules! instruction {
    // Numeric argument
    (@arg $p:ident n) => {
        InstructionArg::Number($p)
    };
    // TODO other arguments...
    ($name:ident, [$(($($args:ident),*)),*], |$($p:ident.$t:ident),*| $body:block) => {
        Instruction::new(
            stringify!($name),
            |args: &[InstructionArg]| match args {
                [$(instruction!(@arg $p $t)),*] => {
                    $body
                }
                _ => panic!("Invalid arguments to instruction!"),
            },
            vec![$(vec![$(stringify!($args)),*]),*]
        ).unwrap()
    };
}
macro_rules! instructions {
    ($(($name:ident, [$(($($args:ident),*)),*], |$($p:ident.$t:ident),*| $body:block)),*$(,)*) => {
    //($(($name:ident, [$(($($args:ident),*)),*], |$($p:ident.$t:ident),*| $body:block)),*) => {
        vec![
            $(instruction!($name, [$(($($args),*)),*], |$($p.$t),*| $body)),*
        ]
    };
}

// Stationeers MIPS instructions
// (see https://stationeering.com/tools/ic)
/*
 *#[rustfmt::skip]
 *lazy_static! {
 *    pub static ref INSTRS_DVIO: Vec<Instruction<'static>> = instructions![
 *        //(bdns,   f!(println!("test")), [d], [r, n]), // (branching)
 *        //(bdnsal, f!(1), [d], [r, n]),
 *        //(bdse,   f!(1), [d], [r, n]),
 *        //(bdseal, f!(1), [d], [r, n]),
 *        //(brdns,  f!(1), [d], [r, n]),
 *        //(brdse,  f!(1), [d], [r, n]),
 *        // TODO the res...
 *    ];
 *    pub static ref INSTRS_FLOW: Vec<Instruction<'static>> = instructions! [
 *        //(j, f!(1), [r, n]),
 *        // TODO the rest...
 *    ];
 *    pub static ref INSTRS_VSEL: Vec<Instruction<'static>> = instructions! [
 *        // TODO the rest...
 *    ];
 *    pub static ref INSTRS_MATH: Vec<Instruction<'static>> = instructions! [
 *        //(add, f!(1), [r], [r, n], [r, n])
 *        (add, [(n), (n)], |a.n, b.n| {
 *            println!("{}", a + b);
 *        }),
 *        (add, [(n), (n)], |a.n, b.n| {
 *            println!("{}", a + b);
 *        }),
 *        // TODO the rest...
 *    ];
 *    pub static ref INSTRS_BOOL: Vec<Instruction<'static>> = instructions! [
 *        // TODO the rest...
 *    ];
 *    pub static ref INSTRS_STCK: Vec<Instruction<'static>> = instructions! [
 *        // TODO the rest...
 *    ];
 *    pub static ref INSTRS_MISC: Vec<Instruction<'static>> = instructions! [
 *        // TODO the rest...
 *    ];
 *}
 */
