#![allow(dead_code, unused_imports, unused_variables, unused_mut, unused_macros)]
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Result as IOResult};
use std::iter::once;
use std::path::Path;

use float_cmp::approx_eq;
use itertools::Itertools;
use maplit::hashmap;
use petgraph::graph::{DiGraph, Graph, NodeIndex};
use regex::Regex;
use std::convert::TryFrom;

//#[macro_use]
//pub mod instruction;
//use instruction::{Instruction, InstructionArg, InstructionArgType};

#[derive(Clone)]
pub enum Device {
    Unset,
    Set(HashMap<String, f32>),
}

// bool: is_default
#[derive(Clone, Copy, Debug)]
pub enum Alias {
    Device(usize, bool),
    Register(usize, bool),
}

pub type ParameterSet = Vec<String>;

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
                    "alias" => instruction!(ic, [t.t, a.a],
                        { ic.add_alias(t, a) }),
                    "add"   => instruction!(ic, [r.r, a.n, b.n],
                        { ic.set_register(r, a + b)? }),
                    "sub"   => instruction!(ic, [r.r, a.n, b.n],
                        { ic.set_register(r, a - b)? }),
                    "yield" => instruction!(ic, [], { ic.halt = true }),
                    "j" => instruction!(ic, [l.n],
                        { ic.next_line = l as usize }),
                    "beq" => instruction!(ic, [a.n, b.n, l.n],
                        { if approx_eq!(f32, a, b) { ic.next_line = l as usize; } }),
                    "bne" => instruction!(ic, [a.n, b.n, l.n],
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
        parameters: &ParameterSet,
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

pub struct ICState {
    // Hard state
    devices: Vec<Device>,
    registers: Vec<f32>,
    aliases: HashMap<String, Alias>,
    definitions: HashMap<String, f32>,
    labels: HashMap<String, usize>,
    // Operation state
    next_line: usize,
    instr_per_tick: usize,
    instr_counter: usize,
    halt: bool,
}

impl ICState {
    pub fn new(ndevices: usize, nregisters: usize, instr_per_tick: usize) -> Self {
        let daliases = (0..ndevices).map(|i| (format!("d{}", i), Alias::Device(i, true)));
        let raliases = (0..nregisters)
            .map(|i| (format!("r{}", i), Alias::Register(i, true)))
            .chain(
                ["ra", "sp"]
                    .iter()
                    .enumerate()
                    .map(|(i, &l)| (l.to_owned(), Alias::Register(i + nregisters + 1, true))),
            );
        Self {
            devices: vec![Device::Unset; ndevices],
            registers: vec![0.0; nregisters + 2],
            aliases: daliases.chain(raliases).collect(),
            definitions: HashMap::new(),
            labels: HashMap::new(),
            next_line: 0,
            instr_per_tick,
            instr_counter: 0,
            halt: false,
        }
    }

    pub fn get_ra(&self) -> f32 {
        self.registers[self.registers.len() - 2]
    }

    pub fn set_ra(&mut self, v: f32) {
        let i = self.registers.len() - 2;
        self.registers[i] = v;
    }

    pub fn get_sp(&self) -> f32 {
        self.registers[self.registers.len() - 1]
    }

    pub fn set_sp(&mut self, v: f32) {
        let i = self.registers.len() - 1;
        self.registers[i] = v;
    }

    pub fn set_register(&mut self, r: Alias, v: f32) -> Result<(), String> {
        if let Some(r) = {
            if let Alias::Register(i, _) = r {
                self.registers.get_mut(i)
            } else {
                None
            }
        } {
            (*r) = v;
            Ok(())
        } else {
            Err("Invalid register index".to_owned())
        }
    }

    pub fn add_alias(&mut self, t: &str, a: Alias) {
        let a = match a {
            Alias::Device(d, _) => Alias::Device(d, false),
            Alias::Register(r, _) => Alias::Register(r, false),
        };
        self.aliases.insert(t.to_owned(), a);
    }

    fn try_alias(&self, token: &str) -> Result<Alias, String> {
        match self.aliases.get(token) {
            Some(a) => Ok(*a),
            _ => Err(format!("'{}' failed as alias", token)),
        }
    }

    fn try_register(&self, token: &str) -> Result<Alias, String> {
        match self.aliases.get(token) {
            Some(Alias::Register(i, f)) => Ok(Alias::Register(*i, *f)),
            _ => Err(format!("'{}' failed to parse as register", token)),
        }
    }

    fn try_number(&self, token: &str) -> Result<f32, String> {
        if let Ok(n) = token.parse::<f32>() {
            Ok(n)
        } else if let Some(Alias::Register(i, _)) = self.aliases.get(token) {
            Ok(self.registers[*i])
        } else {
            Err("Not a number!".to_owned())
        }
    }
}

impl std::fmt::Display for ICState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (i, v) in self
            .registers
            .iter()
            .take(self.registers.len() - 2)
            .enumerate()
        {
            writeln!(f, "{:>3}:{}", format!("r{}", i), v)?;
        }
        writeln!(f, " ra:{}", self.get_ra())?;
        writeln!(f, " sp:{}", self.get_sp())?;
        for (k, v) in self.aliases.iter() {
            match v {
                Alias::Device(i, false) => writeln!(f, "{} -> d{}", k, i)?,
                Alias::Register(i, false) => writeln!(f, "{} -> r{}", k, i)?,
                _ => {}
            };
        }
        Ok(())
    }
}

impl Default for ICState {
    fn default() -> Self {
        Self::new(6, 16, 128)
    }
}

pub fn try_run_line<I: InstructionSet>(
    ic: &mut ICState,
    line: &str,
    instructions: &I,
    parameters: &ParameterSet,
) -> Result<(), String> {
    let mut tokens = line.split(" ");
    if let Some(instr) = tokens.next() {
        let args = tokens.collect();
        instructions.try_run(instr, args, ic, parameters)
    } else {
        // empty line error?
        Err(format!("Empty line?"))
    }
}

pub fn try_run<I: InstructionSet>(
    ic: &mut ICState,
    lines: &Vec<String>,
    instruction: &I,
    parameters: &ParameterSet,
) -> Result<(), String> {
    while !(ic.halt || ic.instr_counter >= ic.instr_per_tick || ic.next_line >= lines.len()) {
        let i = ic.next_line;
        ic.next_line += 1;
        if let Some(line) = lines.get(i) {
            try_run_line(ic, line, instruction, parameters)?;
            ic.instr_counter += 1;
        } else {
            return Err(format!("Line index '{}' out of range", ic.next_line));
        }
    }
    Ok(())
}

fn main() -> IOResult<()> {
    let instructions = StationeersInstructionSet::new();
    let parameters = ParameterSet::default();
    let mut ic = ICState::default();

    let file = File::open("test.mips").unwrap();
    let lines: Vec<String> = BufReader::new(file).lines().try_collect().unwrap();
    let mips = lines.join("\n");

    println!("Running(\n\"\n{}\n\")", mips);

    println!("{:?}", try_run(&mut ic, &lines, &instructions, &parameters));
    print!("{}", ic);
    println!("ic.instr_counter = {}", ic.instr_counter);

    //#[rustfmt::skip]
    //let add = instruction!(add, [(n), (n)], |a.n, b.n| {
    //println!("{}", a + b);
    //});
    //#[rustfmt::skip]
    //(add.f)(&[
    //InstructionArg::Number(2),
    //InstructionArg::Number(3)
    //]);
    // > 5

    //let mut graph = DiGraph::new();
    //let a = graph.add_node(0);

    Ok(())
}
