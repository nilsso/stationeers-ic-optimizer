use std::collections::HashMap;

use float_cmp::approx_eq;
use maplit::hashmap;

use crate::{device::Device, ic::ICState};

pub trait InstructionSet {
    fn try_run(&self, instr_token: &str, args: Vec<&str>, ic: &mut ICState) -> Result<(), String>;
}

pub type Instruction = Box<dyn Fn(&mut ICState, Vec<&str>) -> Result<(), String>>;

pub struct StationeersInstructionSet {
    instructions: HashMap<&'static str, Instruction>,
}

macro_rules! instruction {
    (@arg $ic:ident, $a:ident.a) => {
        $ic.try_alias($a)?;
    };
    (@arg $ic:ident, $a:ident.d) => {
        $ic.try_device($a)?;
    };
    (@arg $ic:ident, $a:ident.r) => {
        $ic.try_register($a)?
    };
    (@arg $ic:ident, $a:ident.n) => {
        $ic.try_number($a)?
    };
    (@arg $ic:ident, $a:ident.l) => {
        $ic.try_line_number($a)?
    };
    (@arg $ic:ident, $a:ident.t) => {
        $a // &str
    };
    ([$($a:ident.$t:tt),*], $ic:ident, $body:expr) => {{
        Box::new(|ic: &mut ICState, args: Vec<&str>| -> Result<(), String> {
            match args.as_slice() {
                [$($a),*] => {
                    let $ic: &mut ICState = ic;
                    $(
                        #[allow(unused_variables)]
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

macro_rules! instructions {
    ($(($name:tt, [$($a:ident.$t:tt),*], $ic:ident, $body:expr)),*$(,)*) => {{
        hashmap! {
            $(stringify!($name) => instruction!([$($a.$t),*], $ic, $body)),*
        }
    }};
}

impl StationeersInstructionSet {
    pub fn new() -> Self {
        Self {
            instructions: instructions! {
                // (<name>, ic, [(<var>.<type>)*], { <expressions> }),
                // where type:
                // * `a` - alias (device or register)
                // * `r` - register (alias)
                // * `n` - a number (either a literal or a register alias)
                // * `t` - token (a alias/define/parameter string)
                // Device IO ----------------------------------------------------------------------
                (bdns,   [d.d, l.l],             ic, { let f = !ic.is_device_set(d)?; ic.branch_helper(l, f, false, false); }),
                (bdnsal, [d.d, l.l],             ic, { let f = !ic.is_device_set(d)?; ic.branch_helper(l, f, false,  true); }),
                (bdse,   [d.d, l.l],             ic, { let f =  ic.is_device_set(d)?; ic.branch_helper(l, f, false, false); }),
                (bdseal, [d.d, l.l],             ic, { let f =  ic.is_device_set(d)?; ic.branch_helper(l, f, false,  true); }),
                (brdns,  [d.d, l.l],             ic, { let f = !ic.is_device_set(d)?; ic.branch_helper(l, f,  true, false); }),
                (brdse,  [d.d, l.l],             ic, { let f =  ic.is_device_set(d)?; ic.branch_helper(l, f,  true, false); }),
                (l,      [r.r, d.d, p.t],        ic, { let v =  ic.try_get_device_param(d, p)?; ic.set_register(r, v)?; }),
                (lb,     [r.r, h.n, p.t, m.n],   ic, {}),
                // Loads reagent of device's reagentMode to register.
                // Contents (0), Required (1), Recipe (2). Can use either the word, or the number.
                (lr,     [r.r, d.d, m.n, p.t],   ic, {}), // TODO
                (ls,     [r.r, d.d, s.n, p.t],   ic, {}), // TODO
                (s,      [d.d, p.t, n.n],        ic, {}), // TODO
                (sb,     [h.n, p.t, n.n],        ic, {}), // TODO

                // Flow Control, Branches and Jumps -----------------------------------------------
                (bap,    [a.n, b.n, c.n, l.l],   ic, {}), // TODO (I think c is epsilon?)
                (bapal,  [a.n, b.n, c.n, l.l],   ic, {}), // TODO
                (bapz,   [a.n, b.n, l.l],        ic, {}), // TODO
                (bapzal, [a.n, b.n, l.l],        ic, {}), // TODO
                (beq,    [a.n, b.n, l.l],        ic, ic.branch_helper(l, approx_eq!(f32, a, b),    false, false)),
                (beqal,  [a.n, b.n, l.l],        ic, ic.branch_helper(l, approx_eq!(f32, a, b),    false,  true)),
                (beqz,   [a.n, l.l],             ic, ic.branch_helper(l, approx_eq!(f32, a, 0.0),  false, false)),
                (beqzal, [a.n, l.l],             ic, ic.branch_helper(l, approx_eq!(f32, a, 0.0),  false,  true)),
                (bge,    [a.n, b.n, l.l],        ic, ic.branch_helper(l, a >= b,                   false, false)),
                (bgeal,  [a.n, b.n, l.l],        ic, ic.branch_helper(l, a >= b,                   false,  true)),
                (bgez,   [a.n, b.n, l.l],        ic, ic.branch_helper(l, a >= 0.0,                 false, false)),
                (bgezal, [a.n, b.n, l.l],        ic, ic.branch_helper(l, a >= 0.0,                 false,  true)),
                (bgt,    [a.n, b.n, l.l],        ic, ic.branch_helper(l, a > b,                    false, false)),
                (bgtal,  [a.n, b.n, l.l],        ic, ic.branch_helper(l, a > b,                    false,  true)),
                (bgtz,   [a.n, b.n, l.l],        ic, ic.branch_helper(l, a > 0.0,                  false, false)),
                (bgtzal, [a.n, b.n, l.l],        ic, ic.branch_helper(l, a > 0.0,                  false,  true)),
                (ble,    [a.n, b.n, l.l],        ic, ic.branch_helper(l, a <= b,                   false, false)),
                (bleal,  [a.n, b.n, l.l],        ic, ic.branch_helper(l, a <= b,                   false,  true)),
                (blez,   [a.n, b.n, l.l],        ic, ic.branch_helper(l, a <= 0.0,                 false, false)),
                (blezal, [a.n, b.n, l.l],        ic, ic.branch_helper(l, a <= 0.0,                 false,  true)),
                (blt,    [a.n, b.n, l.l],        ic, ic.branch_helper(l, a < b,                    false, false)),
                (bltal,  [a.n, b.n, l.l],        ic, ic.branch_helper(l, a < b,                    false,  true)),
                (bltz,   [a.n, b.n, l.l],        ic, ic.branch_helper(l, a < 0.0,                  false, false)),
                (bltzal, [a.n, b.n, l.l],        ic, ic.branch_helper(l, a < 0.0,                  false,  true)),
                (bna,    [a.n, b.n, c.n, l.l],   ic, {}), // TODO
                (bnaal,  [a.n, b.n, c.n, l.l],   ic, {}), // TODO
                (bnaz,   [a.n, b.n, l.l],        ic, {}), // TODO
                (bnazal, [a.n, b.n, l.l],        ic, {}), // TODO
                (bne,    [a.n, b.n, l.l],        ic, ic.branch_helper(l, !approx_eq!(f32, a, b),   false, false)),
                (bneal,  [a.n, b.n, l.l],        ic, ic.branch_helper(l, !approx_eq!(f32, a, b),   false,  true)),
                (bnez,   [a.n, b.n, l.l],        ic, ic.branch_helper(l, !approx_eq!(f32, a, 0.0), false, false)),
                (bnezal, [a.n, b.n, l.l],        ic, ic.branch_helper(l, !approx_eq!(f32, a, 0.0), false,  true)),
                (brap,   [a.n, b.n, c.n, l.l],   ic, {}), // TODO
                (brapz,  [a.n, b.n, l.l],        ic, {}), // TODO
                (breq,   [a.n, b.n, l.l],        ic, ic.branch_helper(l, approx_eq!(f32, a, b),     true, false)),
                (breqz,  [a.n, l.l],             ic, ic.branch_helper(l, approx_eq!(f32, a, 0.0),   true, false)),
                (brge,   [a.n, b.n, l.l],        ic, ic.branch_helper(l, a >= b,                    true, false)),
                (brgez,  [a.n, b.n, l.l],        ic, ic.branch_helper(l, a >= 0.0,                  true, false)),
                (brgt,   [a.n, b.n, l.l],        ic, ic.branch_helper(l, a > b,                     true, false)),
                (brgtz,  [a.n, b.n, l.l],        ic, ic.branch_helper(l, a > 0.0,                   true, false)),
                (brle,   [a.n, b.n, l.l],        ic, ic.branch_helper(l, a <= b,                    true, false)),
                (brlez,  [a.n, b.n, l.l],        ic, ic.branch_helper(l, a <= 0.0,                  true, false)),
                (brlt,   [a.n, b.n, l.l],        ic, ic.branch_helper(l, a < b,                     true, false)),
                (brltz,  [a.n, b.n, l.l],        ic, ic.branch_helper(l, a < 0.0,                   true, false)),
                (brna,   [a.n, b.n, c.n, l.l],   ic, {}), // TODO
                (brnaz,  [a.n, b.n, l.l],        ic, {}), // TODO
                (brne,   [a.n, b.n, c.n, l.l],   ic, {}), // TODO
                (brnez,  [a.n, b.n, l.l],        ic, {}), // TODO
                (j,      [l.l],                  ic, ic.branch_helper(l, true,                     false, false)),
                (jal,    [l.l],                  ic, ic.branch_helper(l, true,                     false,  true)),
                (jr,     [l.l],                  ic, ic.branch_helper(l, true,                      true, false)),

                // Variable Selection -------------------------------------------------------------
                (sap,    [r.r, a.n, b.n, c.n],   ic, {}), // TODO
                (sapz,   [r.r, a.n, b.n],        ic, {}), // TODO
                (sdns,   [r.r, d.d],             ic, { let v = !ic.is_device_set(d)?; ic.set_register(r, if v { 1.0 } else { 0.0 })?; }),
                (sdse,   [r.r, d.d],             ic, { let v =  ic.is_device_set(d)?; ic.set_register(r, if v { 1.0 } else { 0.0 })?; }),
                (select, [r.r, a.n, b.n, c.n],   ic, ic.set_register(r, if approx_eq!(f32, a, 0.0) { b } else { c })?),
                (seq,    [r.r, a.n, b.n],        ic, ic.set_register(r, if approx_eq!(f32, a, b) { 1.0 } else { 0.0 })?),
                (seqz,   [r.r, a.n],             ic, ic.set_register(r, if approx_eq!(f32, a, 0.0) { 1.0 } else { 0.0 })?),
                (sge,    [r.r, a.n, b.n],        ic, ic.set_register(r, if a >= b { 1.0 } else { 0.0 })?),
                (sgez,   [r.r, a.n],             ic, ic.set_register(r, if a >= 0.0 { 1.0 } else { 0.0 })?),
                (sgt,    [r.r, a.n, b.n],        ic, ic.set_register(r, if a > b { 1.0 } else { 0.0 })?),
                (sgtz,   [r.r, a.n],             ic, ic.set_register(r, if a > 0.0 { 1.0 } else { 0.0 })?),
                (sle,    [r.r, a.n, b.n],        ic, ic.set_register(r, if a <= b { 1.0 } else { 0.0 })?),
                (slez,   [r.r, a.n],             ic, ic.set_register(r, if a <= 0.0 { 1.0 } else { 0.0 })?),
                (slt,    [r.r, a.n, b.n],        ic, ic.set_register(r, if a < b { 1.0 } else { 0.0 })?),
                (sltz,   [r.r, a.n],             ic, ic.set_register(r, if a < 0.0 { 1.0 } else { 0.0 })?),
                (sna,    [r.r, a.n, b.n, c.n],   ic, {}), // TODO
                (snaz,   [r.r, a.n, b.n],        ic, {}), // TODO
                // Register = 1 if a != b, otherwise 0
                (sne,    [r.r, a.n, b.n, c.n],   ic, {}), // TODO
                // Register = 1 if a != 0, otherwise 0
                (snez,   [r.r, a.n, b.n],        ic, {}), // TODO

                // Mathematical Operations --------------------------------------------------------
                (abs,    [r.r, a.n],             ic, ic.set_register(r, a.abs())?),
                (acos,   [r.r, a.n],             ic, ic.set_register(r, a.acos())?),
                (add,    [r.r, a.n, b.n],        ic, ic.set_register(r, a + b)?),
                (asin,   [r.r, a.n],             ic, ic.set_register(r, a.asin())?),
                (atan,   [r.r, a.n],             ic, ic.set_register(r, a.atan())?),
                (ceil,   [r.r, a.n],             ic, ic.set_register(r, a.ceil())?),
                (cos,    [r.r, a.n],             ic, ic.set_register(r, a.cos())?),
                (div,    [r.r, a.n, b.n],        ic, ic.set_register(r, a / b)?),
                (exp,    [r.r, a.n],             ic, ic.set_register(r, a.exp())?),
                (floor,  [r.r, a.n],             ic, ic.set_register(r, a.floor())?),
                (log,    [r.r, a.n],             ic, ic.set_register(r, a.ln())?),
                (max,    [r.r, a.n, b.n],        ic, ic.set_register(r, f32::max(a, b))?),
                (min,    [r.r, a.n, b.n],        ic, ic.set_register(r, f32::min(a, b))?),
                (mod,    [r.r, a.n, b.n],        ic, ic.set_register(r, a % b)?),
                (mul,    [r.r, a.n, b.n],        ic, ic.set_register(r, a * b)?),
                (rand,   [r.r],                  ic, ic.set_register(r, rand::random::<f32>())?),
                (round,  [r.r, a.n],             ic, ic.set_register(r, a.round())?),
                (sin,    [r.r, a.n],             ic, ic.set_register(r, a.sin())?),
                (sqrt,   [r.r, a.n],             ic, ic.set_register(r, a.sqrt())?),
                (sub,    [r.r, a.n, b.n],        ic, ic.set_register(r, a - b)?),
                (tan,    [r.r, a.n],             ic, ic.set_register(r, a.tan())?),
                (trunc,  [r.r, a.n],             ic, ic.set_register(r, a.trunc())?),

                // Logic --------------------------------------------------------------------------
                (and,    [r.r, a.n, b.n],        ic, ic.set_register(r, if a > 0.0 && b > 0.0 { 1.0 } else { 0.0 })?),
                (nor,    [r.r, a.n, b.n],        ic, ic.set_register(r, if !(a > 0.0 || b > 0.0) { 1.0 } else { 0.0 })?),
                (or,     [r.r, a.n, b.n],        ic, ic.set_register(r, if a > 0.0 || b > 0.0 { 1.0 } else { 0.0 })?),
                (xor,    [r.r, a.n, b.n],        ic, ic.set_register(r, {
                    let a = a > 0.0;
                    let b = b > 0.0;
                    if a || b && !(a && b) { 1.0 } else { 0.0 }
                })?),

                // Stack --------------------------------------------------------------------------
                (peek,   [r.r],                  ic, { let v = ic.try_peek()?; ic.set_register(r, v)?; }), // TODO
                (pop,    [r.r],                  ic, { let v = ic.try_pop()?; ic.set_register(r, v)?; }), // TODO
                (push,   [a.n],                  ic, ic.try_push(a)?), // TODO

                // Misc ---------------------------------------------------------------------------
                (alias,  [t.t, a.a],             ic, ic.add_alias(t, a)),
                (define, [t.t, n.n],             ic, ic.add_definition(t, n)),
                (hcf,    [],                     ic, ic.halt = true), // TODO maybe do something fun instead
                (move,   [r.r, n.n],             ic, ic.set_register(r, n)?),
                (sleep,  [n.n],                  ic, ic.halt = true),
                (yield,  [],                     ic, ic.halt = true),
            },
        }
    }
}

impl InstructionSet for StationeersInstructionSet {
    fn try_run(&self, instr_token: &str, args: Vec<&str>, ic: &mut ICState) -> Result<(), String> {
        if let Some(instr) = self.instructions.get(instr_token) {
            return instr(ic, args);
        }
        Err(format!("Unrecognized instruction '{}'!", instr_token))
    }
}
