#![allow(dead_code)]
//#![allow(unused_imports)]
//#![allow(unused_variables)]
//#![allow(unused_mut)]
//#![allow(unused_macros)]

use std::fs::File;
use std::io::{BufRead, BufReader, Result as IOResult};

use itertools::Itertools;
//use petgraph::graph::{DiGraph, Graph, NodeIndex};

pub mod alias;
pub mod device;
pub mod ic;
pub mod instruction;

use crate::{
    device::ParameterSet,
    ic::ICState,
    instruction::{InstructionSet, StationeersInstructionSet},
};

/// Parse and run a single instruction line
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

/// Parse and run lines of instructions
pub fn try_run<I: InstructionSet>(
    ic: &mut ICState,
    lines: &Vec<String>,
    instruction: &I,
    parameters: &ParameterSet,
) -> Result<(), String> {
    // Run while:
    // - no halt instruction was given (a.k.a. "yield")
    // - the instructions per tick amount has yet to be reached
    // - we're not at the end of the file
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

    //let mut graph = DiGraph::new();
    //let a = graph.add_node(0);

    Ok(())
}
