use crate::mypthreads::MyMutex;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleType {
    Car,
    Ambulance,
    Boat,
    SupplyTruck,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeType {
    TrafficLight,  // Puente 1
    YieldSign,     // Puente 2  
    TwoLanes,      // Puente 3
}

#[derive(Debug, Clone)]
pub struct Vehicle {
    pub id: usize,
    pub vtype: VehicleType,
    pub pos: (usize, usize),
    pub dest: (usize, usize),
}

use std::sync::{Arc};

#[derive(Debug)]
pub struct Bridge {
    pub id: usize,
    pub name: String,
    pub bridge_type: BridgeType,           
    pub mutex: Arc<MyMutex>,
    pub is_blocked: Arc<std::sync::Mutex<bool>>,
    pub green_light: Arc<std::sync::Mutex<bool>>,   
}
