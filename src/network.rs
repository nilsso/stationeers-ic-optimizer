use std::collections::HashMap;

use crate::device::DeviceType;

type NetworkDeviceId = usize;

struct Network {
    devices: HashMap<String, Vec<DeviceType>>,
}

impl Network {
    //pub fn add_device(&mut self,
}
