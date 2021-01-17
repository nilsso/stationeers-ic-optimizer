use std::collections::HashMap;

use crate::{alias::Alias, device::Device};

pub struct ICState {
    // Hard state
    devices: Vec<Device>,
    registers: Vec<f32>,
    aliases: HashMap<String, Alias>,
    definitions: HashMap<String, f32>,
    labels: HashMap<String, usize>,
    // Operation state
    pub instr_per_tick: usize,
    pub instr_counter: usize,
    pub next_line: usize,
    pub halt: bool,
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

    pub fn try_alias(&self, token: &str) -> Result<Alias, String> {
        match self.aliases.get(token) {
            Some(a) => Ok(*a),
            _ => Err(format!("'{}' failed as alias", token)),
        }
    }

    pub fn try_register(&self, token: &str) -> Result<Alias, String> {
        match self.aliases.get(token) {
            Some(Alias::Register(i, f)) => Ok(Alias::Register(*i, *f)),
            _ => Err(format!("'{}' failed to parse as register", token)),
        }
    }

    pub fn try_number(&self, token: &str) -> Result<f32, String> {
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
