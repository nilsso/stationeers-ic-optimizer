#![allow(dead_code, unused_imports, unused_variables, unused_mut, unused_macros)]
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Result as IOResult};
use std::iter::once;
use std::path::Path;

use petgraph::graph::{DiGraph, Graph, NodeIndex};

#[macro_use]
pub mod instruction;
use instruction::{Instruction, InstructionArg, InstructionArgType};

/*
 *#[derive(Debug)]
 *enum TokenType {
 *    Uncategorized,
 *    Assignment,
 *    Instruction,
 *    Device,
 *    Register,
 *    Parameter,
 *    Label,
 *    Comment,
 *}
 *
 *#[derive(Debug)]
 *struct Token {
 *    raw: String,
 *    token_type: TokenType,
 *}
 *
 *impl Token {
 *    fn new(raw: &str) -> Self {
 *        Self {
 *            raw: raw.to_owned(),
 *            token_type: TokenType::Uncategorized,
 *        }
 *    }
 *}
 *
 *fn parse_token(token: &Token) {
 *    match token.raw.as_str() {
 *        "define" => {}
 *        "alias" => {}
 *        _ => {}
 *    }
 *}
 *
 *#[derive(Debug)]
 *enum Node {
 *    Register(String),
 *    Device(String),
 *    Definition,
 *}
 *
 *struct Parser {
 *    tokens: Vec<Token>,
 *    graph: Graph<u8, u8>,
 *    aliases: HashMap<String, NodeIndex>,
 *}
 *
 *impl Parser {
 *    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
 *        let file = File::open(path)?;
 *        let lines: Vec<String> = BufReader::new(file)
 *            .lines()
 *            .into_iter()
 *            .filter_map(|l| l.ok())
 *            .collect();
 *        let tokens: Vec<Token> = lines
 *            .iter()
 *            .flat_map(|l| l.split(' ').map(|t| Token::new(t)))
 *            .collect();
 *        Ok(Self {
 *            tokens,
 *            ..Default::default()
 *        })
 *    }
 *
 *    fn parse(&mut self) {
 *        //for n in self.nodes.iter() {
 *        //println!("{:?}", n);
 *        //}
 *        //for t in self.tokens.iter() {
 *        //println!("{:?}", t);
 *        //}
 *        //let mut stack = vec![];
 *        //let mut aliases = vec![];
 *    }
 *}
 *
 *impl Default for Parser {
 *    fn default() -> Self {
 *        let mut aliases = HashMap::new();
 *        //for (0..6).map(|i| Node::Device
 *        //let devices = (0..6).map(|i| Node::Device(format!("d{}", i)));
 *        //let registers = (0..16)
 *        //.map(|i| Node::Register(format!("r{}", i)))
 *        //.chain(["ra", "sp"].iter().map(|l| Node::Register(l.to_string())));
 *        Self {
 *            tokens: vec![],
 *            //nodes: devices.chain(registers).collect(),
 *            graph: Graph::new(),
 *            aliases,
 *        }
 *    }
 *}
 */

#[derive(Clone)]
enum Device {
    Unset,
    Set(HashMap<String, f32>),
}

enum Alias {
    Device(usize),
    Register(usize),
}

struct ICState {
    devices: Vec<Device>,
    registers: Vec<f32>,
    aliases: HashMap<String, Alias>,
}

impl ICState {
    pub fn new(ndevices: usize, nregisters: usize) -> Self {
        let daliases = (0..ndevices).map(|i| (format!("d{}", i), Alias::Device(i)));
        let raliases = (0..nregisters)
            .map(|i| (format!("r{}", i), Alias::Register(i)))
            .chain(
                ["ra", "sp"]
                    .iter()
                    .enumerate()
                    .map(|(i, &l)| (l.to_owned(), Alias::Register(i + nregisters + 1))),
            );
        Self {
            devices: vec![Device::Unset; ndevices],
            registers: vec![0.0; nregisters],
            aliases: daliases.chain(raliases).collect(),
        }
    }
}

impl Default for ICState {
    fn default() -> Self {
        Self::new(6, 16)
    }
}

#[derive(Debug)]
enum Arg {
    Device(usize),
    Register(usize),
    Number(f32),
    Parameter(String),
}

impl Arg {
    fn to_number(self, ic: &ICState) -> Result<f32, String> {
        match self {
            Arg::Register(i) => match ic.registers.get(i) {
                Some(n) => Ok(*n),
                None => Err(format!("Index {} out of register range!", i)),
            },
            Arg::Number(n) => Ok(n),
            _ => Err(format!("Can't convert {:?} to number!", self)),
        }
    }
}

fn main() -> IOResult<()> {
    let mut ic = ICState::default();

    ic.registers[0] = 3.14;

    let f = |ic: &mut ICState| {};
    f(&mut ic);

    //let arg = Arg::Device(0);
    let arg = Arg::Register(0);
    let arg = Arg::Number(8.314);
    //let arg = Arg::Parameter("On".to_owned());

    match arg.to_number(&ic) {
        Ok(n) => println!("{}", n),
        Err(e) => println!("Error: {}", e),
    };

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
