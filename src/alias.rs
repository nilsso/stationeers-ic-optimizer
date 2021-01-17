// bool: is_default
#[derive(Clone, Copy, Debug)]
pub enum Alias {
    Device(usize, bool),
    Register(usize, bool),
}
