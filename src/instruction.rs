use std::collections::HashMap;

use float_cmp::approx_eq;
use maplit::hashmap;
use regex::Regex;

use crate::{device::ParameterSet, ic::ICState};

pub trait InstructionSet {
    fn try_run(
        &self,
        instr_token: &str,
        args: Vec<&str>,
        ic: &mut ICState,
        parameters: &ParameterSet,
    ) -> Result<(), String>;
}

pub type Instruction = Box<dyn Fn(&mut ICState, Vec<&str>) -> Result<(), String>>;

pub struct StationeersInstructionSet {
    groups: Vec<(Regex, HashMap<&'static str, Instruction>)>,
    singles: HashMap<&'static str, Instruction>,
}

macro_rules! instruction {
    (@arg $ic:ident, $a:ident.a) => {
        $ic.try_alias($a)?;
    };
    (@arg $ic:ident, $a:ident.r) => {
        $ic.try_register($a)?
    };
    (@arg $ic:ident, $a:ident.n) => {
        $ic.try_number($a)?
    };
    (@arg $ic:ident, $a:ident.t) => {
        $a // &str
    };
    ($ic:ident, [$($a:ident.$t:tt),*], $body:expr) => {{
        Box::new(|ic: &mut ICState, args: Vec<&str>| -> Result<(), String> {
            match args.as_slice() {
                [$($a),*] => {
                    let $ic: &mut ICState = ic;
                    $(
                        let $a = instruction!(@arg $ic, $a.$t);
                    )*
                    $body;
                    Ok(())
                }
                _ => Err("Failed for arguments".to_owned()),
            }
        }) as Instruction
    }};
}

impl StationeersInstructionSet {
    pub fn new() -> Self {
        Self {
            singles: hashmap! {
                 // "<name>" => instruction!(ic,
                 //     [(<var>.<type>)*],
                 //     { <expressions> }),
                 // where type:
                 // * `a` - alias (device or register)
                 // * `r` - register (alias)
                 // * `n` - a number (either a literal or a register alias)
                 // * `t` - token (a alias/define/parameter string)
                    "alias" => instruction!(ic, [t.t, a.a],
                        { ic.add_alias(t, a) }),
                    "add"   => instruction!(ic, [r.r, a.n, b.n],
                        { ic.set_register(r, a + b)? }),
                    "sub"   => instruction!(ic, [r.r, a.n, b.n],
                        { ic.set_register(r, a - b)? }),
                    "yield" => instruction!(ic, [],
                        { ic.halt = true }),
                    "j"     => instruction!(ic, [l.n],
                        { ic.next_line = l as usize }),
                    "beq"   => instruction!(ic, [a.n, b.n, l.n],
                        { if approx_eq!(f32, a, b) { ic.next_line = l as usize; } }),
                    "bne"   => instruction!(ic, [a.n, b.n, l.n],
                        { if !approx_eq!(f32, a, b) { ic.next_line = l as usize } })
            },
            groups: vec![],
        }
    }
}

impl InstructionSet for StationeersInstructionSet {
    fn try_run(
        &self,
        instr_token: &str,
        args: Vec<&str>,
        ic: &mut ICState,
        _parameters: &ParameterSet,
    ) -> Result<(), String> {
        if let Some(instr) = self.singles.get(instr_token) {
            return instr(ic, args);
        } else {
            for (p, singles) in self.groups.iter() {
                if p.is_match(instr_token) {
                    if let Some(instr) = singles.get(instr_token) {
                        return instr(ic, args);
                    }
                }
            }
        }
        Err(format!("Unrecognized instruction '{}'!", instr_token))
    }
}
