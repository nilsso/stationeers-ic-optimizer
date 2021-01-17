use std::collections::HashMap;

pub type ParameterSet = Vec<String>;

#[derive(Clone)]
pub enum Device {
    Unset,
    Set(HashMap<String, f32>),
}
