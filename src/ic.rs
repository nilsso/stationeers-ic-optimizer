use std::collections::HashMap;

//use rand:: w

use crate::{
    alias::Alias,
    device::{Device, DeviceType},
};

pub struct ICState {
    // Hard state
    devices: Vec<Device>,
    registers: Vec<f32>,
    aliases: HashMap<String, Alias>,
    definitions: HashMap<String, f32>,
    labels: HashMap<String, usize>,
    stack: Vec<f32>,
    // Operation state
    pub instr_per_tick: usize,
    pub instr_counter: usize,
    pub next_line: usize,
    pub halt: bool,
}

// Most public functions for ICState are helper functions for writing
// instructions sets.
impl ICState {
    pub fn new(
        ndevices: usize,
        nregisters: usize,
        stack_size: usize,
        instr_per_tick: usize,
    ) -> Self {
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
            stack: vec![0.0; stack_size],
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

    /// Modifies the next line for parsing by an absolute or relative value.
    ///
    /// * `l` - Absolute or relative line number
    /// * `f` - If false nothing is changed
    /// * `relative` - If false, next line number is set to `l` absolute,
    ///   else `l` is added to the current line number.
    /// * `save` - If true, register `ra` is assigned the next line number
    pub fn branch_helper(&mut self, l: usize, f: bool, relative: bool, save: bool) {
        let l = l as f32;
        if save {
            self.set_ra(self.next_line as f32);
        }
        if f {
            self.next_line = if relative {
                l - 1.0 + self.next_line as f32
            } else {
                l
            } as usize;
        }
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

    pub fn get_device(&mut self, d: Alias) -> Result<&Device, String> {
        if let Alias::Device(i, _) = d {
            if let Some(d) = self.devices.get(i) {
                Ok(d)
            } else {
                Err("Invalid device alias".to_owned())
            }
        } else {
            Err(format!("'{:?}' a device alias", d))
        }
    }

    pub fn is_device_set(&mut self, d: Alias) -> Result<bool, String> {
        Ok(matches!(self.get_device(d)?, Device::Set(_)))
    }

    pub fn add_alias(&mut self, t: &str, a: Alias) {
        let a = match a {
            Alias::Device(d, _) => Alias::Device(d, false),
            Alias::Register(r, _) => Alias::Register(r, false),
        };
        self.aliases.insert(t.to_owned(), a);
    }

    pub fn add_definition(&mut self, t: &str, n: f32) {
        self.definitions.insert(t.to_owned(), n);
    }

    pub fn add_label(&mut self, t: &str, n: usize) {
        self.labels.insert(t.to_owned(), n);
    }

    pub fn try_alias(&self, token: &str) -> Result<Alias, String> {
        match self.aliases.get(token) {
            Some(a) => Ok(*a),
            _ => Err(format!("'{}' failed alias lookup", token)),
        }
    }

    pub fn try_device(&self, token: &str) -> Result<Alias, String> {
        match self.aliases.get(token) {
            Some(a) => Ok(*a),
            _ => Err(format!("'{}' failed device lookup", token)),
        }
    }

    pub fn try_register(&self, token: &str) -> Result<Alias, String> {
        match self.aliases.get(token) {
            Some(Alias::Register(i, f)) => Ok(Alias::Register(*i, *f)),
            _ => Err(format!("'{}' failed register lookup", token)),
        }
    }

    pub fn try_number(&self, token: &str) -> Result<f32, String> {
        if let Some(Alias::Register(i, _)) = self.aliases.get(token) {
            Ok(self.registers[*i])
        } else if let Ok(n) = token.parse::<f32>() {
            Ok(n)
        } else if let Some(n) = self.definitions.get(token) {
            Ok(*n)
        } else {
            Err("Not a number!".to_owned())
        }
    }

    pub fn try_line_number(&self, token: &str) -> Result<usize, String> {
        if let Some(Alias::Register(i, _)) = self.aliases.get(token) {
            Ok(self.registers[*i] as usize)
        } else if let Ok(n) = token.parse::<f32>() {
            Ok(n as usize)
        } else if let Some(n) = self.labels.get(token) {
            Ok(*n)
        } else {
            Err("Not a line number!".to_owned())
        }
    }

    pub fn try_set_device(&mut self, a: Alias, dt: DeviceType) -> Result<(), String> {
        if let Some(d) = self.devices.get_mut(a.device_index()?) {
            *d = Device::Set(dt);
            Ok(())
        } else {
            Err(format!("Invalid device alias '{:?}'", a))
        }
    }

    pub fn try_get_device_param(&self, a: Alias, p: &str) -> Result<f32, String> {
        if let Some(Device::Set(dt)) = self.devices.get(a.device_index()?) {
            Ok(dt.get_param(p))
        } else {
            Err(format!("Invalid device alias '{:?}'", a))
        }
    }

    pub fn try_set_device_param(&mut self, a: Alias, p: &str, v: f32) -> Result<(), String> {
        if let Some(Device::Set(dt)) = self.devices.get_mut(a.device_index()?) {
            dt.set_param(p, v);
            Ok(())
        } else {
            Err(format!("Invalid device alias '{:?}'", a))
        }
    }

    fn validate_sp(&self) -> bool {
        let i = self.get_sp(); // TODO: validate mantisa
        i < 0.0 || i >= self.stack.len() as f32
    }

    pub fn try_peek(&self) -> Result<f32, String> {
        let i = self.get_sp();
        if self.validate_sp() {
            Err(format!("Stack index '{}' out of pop range", i))
        } else {
            Ok(self.stack[i as usize])
        }
    }

    pub fn try_pop(&mut self) -> Result<f32, String> {
        let i = self.get_sp();
        if self.validate_sp() {
            Err(format!("Stack index '{}' out of pop range", i))
        } else {
            self.set_sp(i - 1.0);
            Ok(self.stack[i as usize])
        }
    }

    pub fn try_push(&mut self, n: f32) -> Result<(), String> {
        let i = self.get_sp(); // TODO: validate mantisa
        if self.validate_sp() {
            Err(format!("Stack index '{}' out of pop range", i))
        } else {
            self.set_sp(i - 1.0);
            self.stack[i as usize] = n;
            Ok(())
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
        for (k, v) in self.definitions.iter() {
            writeln!(f, "{} = {}", k, v)?;
        }
        for (k, v) in self.labels.iter() {
            writeln!(f, "{} -> {}", k, v)?;
        }
        Ok(())
    }
}

impl Default for ICState {
    fn default() -> Self {
        Self::new(6, 16, 512, 128)
    }
}
