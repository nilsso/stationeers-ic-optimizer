// bool: is_default
#[derive(Clone, Copy, Debug)]
pub enum Alias {
    Device(usize, bool),
    Register(usize, bool),
}

impl Alias {
    pub fn device_index(&self) -> Result<usize, String> {
        match self {
            Alias::Device(i, _) => Ok(*i),
            _ => Err(format!("Cannot get device index from '{:?}'", self)),
        }
    }

    pub fn register_index(&self) -> Result<usize, String> {
        match self {
            Alias::Register(i, _) => Ok(*i),
            _ => Err(format!("Cannot get register index from '{:?}'", self)),
        }
    }
}
