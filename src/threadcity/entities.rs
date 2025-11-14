// entities.rs - very small types

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleType {
    Car,
    Ambulance,
    Boat,
    SupplyTruck,
}

#[derive(Debug, Clone)]
pub struct Vehicle {
    pub id: usize,
    pub vtype: VehicleType,
    pub pos: (usize, usize),
    pub dest: (usize, usize),
}

use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Bridge {
    pub id: usize,
    pub name: String,
    pub mutex: Arc<Mutex<()>>,
}
