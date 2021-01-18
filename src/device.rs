use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct DeviceType {
    name: String,
    parameters: HashMap<String, f32>,
}

impl DeviceType {
    pub fn get_param(&self, p: &str) -> f32 {
        *self.parameters.get(p).unwrap_or(&0.0)
    }

    pub fn set_param(&mut self, p: &str, v: f32) {
        self.parameters.insert(p.to_owned(), v);
    }
}

#[derive(Clone)]
pub enum Device {
    Unset,
    Set(DeviceType),
}
