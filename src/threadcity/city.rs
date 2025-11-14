// city.rs - tiny city model with very simple movement logic
use crate::mypthreads::MyMutex;
use crate::mypthreads::{my_mutex_lock, my_mutex_unlock, MyThreadId};
use std::sync::{Arc};
use crate::threadcity::entities::{Vehicle, VehicleType, Bridge, BridgeType};

#[derive(Debug)]
pub struct City {
    width: usize,
    height: usize,
    vehicles: Vec<Vehicle>,
    next_id: usize,
    pub bridges: Vec<Bridge>,
}

impl City {
    pub fn new(width: usize, height: usize) -> Self {
        let bridges = vec![
    Bridge {
        id: 1,
        name: "Puente Norte".into(),
        bridge_type: BridgeType::TrafficLight,
        mutex: Arc::new(MyMutex::new()),
        is_blocked: Arc::new(std::sync::Mutex::new(false)),
        green_light: Arc::new(std::sync::Mutex::new(true)),
    },
    Bridge {
        id: 2,
        name: "Puente Central".into(),
        bridge_type: BridgeType::YieldSign,
        mutex: Arc::new(MyMutex::new()),
        is_blocked: Arc::new(std::sync::Mutex::new(false)),
        green_light: Arc::new(std::sync::Mutex::new(true)),
    },
    Bridge {
        id: 3,
        name: "Puente Sur".into(),
        bridge_type: BridgeType::TwoLanes,
        mutex: Arc::new(MyMutex::new()),
        is_blocked: Arc::new(std::sync::Mutex::new(false)),
        green_light: Arc::new(std::sync::Mutex::new(true)),
    },
];

        Self {
            width,
            height,
            vehicles: Vec::new(),
            next_id: 0,
            bridges,
        }
    }

    pub fn spawn_vehicle(&mut self, start: (usize, usize), dest: (usize, usize), vtype: VehicleType) {
        let v = Vehicle {
            id: self.next_id,
            vtype,
            pos: start,
            dest,
        };
        self.next_id += 1;
        self.vehicles.push(v);
    }
// step() devuelve un bool
    pub fn step(&mut self, tid: MyThreadId) -> bool {
    let mut all_arrived = true;
    let mut crossings = Vec::new(); // 👈 aquí guardaremos qué vehículos deben cruzar

    for v in &mut self.vehicles {
        if v.pos != v.dest {
            all_arrived = false;
            let (x, y) = v.pos;
            let (dx, dy) = v.dest;
            let new_x = if x < dx { x + 1 } else if x > dx { x - 1 } else { x };
            let new_y = if y < dy { y + 1 } else if y > dy { y - 1 } else { y };
            v.pos = (new_x, new_y);

            // 🚦 Solo guardamos qué puente debe cruzar
            if new_y == 1 {
                crossings.push((v.vtype, 1));
            } else if new_y == 2 {
                crossings.push((v.vtype, 2));
            } else if new_y == 3 {
                crossings.push((v.vtype, 3));
            }
        }
    }

    // 🚗 Ya fuera del for (cuando no hay préstamos mutables)
    // ejecutamos los cruces
    for (vtype, bridge_id) in crossings {
    self.cross_bridge(vtype, bridge_id, tid);  // Use bridge_id, not hardcoded 1
}

    all_arrived
}



    pub fn print_state(&self) {
        // print a simple grid with vehicles marked
        let mut grid = vec![vec!['.'; self.width]; self.height];
        for v in &self.vehicles {
            let (x, y) = v.pos;
            if x < self.width && y < self.height {
                grid[y][x] = match v.vtype {
                    VehicleType::Car => 'C',
                    VehicleType::Ambulance => 'A',
                    VehicleType::Boat => 'B',
                    VehicleType::SupplyTruck => 'S',
                };
            }
        }
        println!("┌{}┐", "─".repeat(self.width));
    for row in &grid {
        let line: String = row.iter().collect();
        println!("│{}│", line);
    }
    println!("└{}┘", "─".repeat(self.width));
}

pub fn cross_bridge(&self, vehicle_type: VehicleType, bridge_id: usize, tid: MyThreadId) {
    let bridge = &self.bridges[bridge_id - 1];
    
    println!("{} quiere cruzar el {}", 
        match vehicle_type {
            VehicleType::Ambulance => "🚑 Ambulancia",
            VehicleType::Car => "🚗 Auto",
            VehicleType::Boat => "🛥️ Barco",
            VehicleType::SupplyTruck => "🚚 Camión",
        },
        bridge.name
    );

    // Different logic based on bridge type
    match bridge.bridge_type {
        BridgeType::TrafficLight => {
            // BRIDGE 1: Traffic light + 1 lane
            // Ambulances get immediate priority
            if vehicle_type == VehicleType::Ambulance {
                my_mutex_lock(&bridge.mutex, tid);
                println!("🚑 Ambulancia cruzando {} (PRIORIDAD)", bridge.name);
                std::thread::sleep(std::time::Duration::from_millis(500));
                my_mutex_unlock(&bridge.mutex, tid).unwrap();
                println!("🚑 Ambulancia salió del {}", bridge.name);
                return;
            }

            // Wait for green light
            loop {
                let green = *bridge.green_light.lock().unwrap();
                if green {
                    break;
                }
                println!("🔴 {} esperando luz verde", 
                    match vehicle_type {
                        VehicleType::Car => "Auto",
                        VehicleType::Boat => "Barco",
                        VehicleType::SupplyTruck => "Camión",
                        _ => "Vehículo",
                    });
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            my_mutex_lock(&bridge.mutex,tid);
            println!("🟢 {} cruzando {} (luz verde)", 
                match vehicle_type {
                    VehicleType::Car => "Auto",
                    VehicleType::Boat => "Barco",
                    VehicleType::SupplyTruck => "Camión",
                    _ => "Vehículo",
                },
                bridge.name
            );
            std::thread::sleep(std::time::Duration::from_millis(800));
            let _ = my_mutex_unlock(&bridge.mutex, tid);
            println!("✅ Salió del {}", bridge.name);
        }

        BridgeType::YieldSign => {
            // BRIDGE 2: Yield sign + 1 lane
            // Ambulances get priority
            if vehicle_type == VehicleType::Ambulance {
                my_mutex_lock(&bridge.mutex, tid);
                println!("🚑 Ambulancia cruzando {} (PRIORIDAD)", bridge.name);
                std::thread::sleep(std::time::Duration::from_millis(500));
                 my_mutex_unlock(&bridge.mutex, tid).unwrap();
                println!("🚑 Ambulancia salió del {}", bridge.name);
                return;
            }

            // Yield = small delay before trying to cross
            println!("⚠️ {} cediendo el paso en {}", 
                match vehicle_type {
                    VehicleType::Car => "Auto",
                    VehicleType::Boat => "Barco",
                    VehicleType::SupplyTruck => "Camión",
                    _ => "Vehículo",
                },
                bridge.name
            );
            std::thread::sleep(std::time::Duration::from_millis(200));

            my_mutex_lock(&bridge.mutex,tid);
            println!("➡️ {} cruzando {}", 
                match vehicle_type {
                    VehicleType::Car => "Auto",
                    VehicleType::Boat => "Barco",
                    VehicleType::SupplyTruck => "Camión",
                    _ => "Vehículo",
                },
                bridge.name
            );
            std::thread::sleep(std::time::Duration::from_millis(800));
            let _ = my_mutex_unlock(&bridge.mutex, tid);
            println!("✅ Salió del {}", bridge.name);
        }

        BridgeType::TwoLanes => {
            // BRIDGE 3: 2 lanes + boats block traffic
            
            // If it's a boat, BLOCK the bridge
            if vehicle_type == VehicleType::Boat {
                println!("⛵ Barco acercándose - BLOQUEANDO {}", bridge.name);
                
                // Block the bridge
                *bridge.is_blocked.lock().unwrap() = true;
                
                my_mutex_lock(&bridge.mutex, tid);
                println!("🚢 Barco pasando bajo {} (puente BLOQUEADO)", bridge.name);
                std::thread::sleep(std::time::Duration::from_millis(2000));
                
                // Unblock
                my_mutex_unlock(&bridge.mutex, tid).unwrap();
                println!("✅ Barco pasó - {} libre nuevamente", bridge.name);
                return;
            }

            // Wait if blocked by boat
            loop {
                let blocked = *bridge.is_blocked.lock().unwrap();
                if !blocked {
                    break;
                }
                println!("🛑 {} esperando - {} bloqueado por barco", 
                    match vehicle_type {
                        VehicleType::Car => "Auto",
                        VehicleType::Ambulance => "Ambulancia",
                        VehicleType::SupplyTruck => "Camión",
                        _ => "Vehículo",
                    },
                    bridge.name
                );
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            // Ambulances still get priority
            if vehicle_type == VehicleType::Ambulance {
                println!("🚑 Ambulancia cruzando {} (PRIORIDAD, 2 carriles)", bridge.name);
                std::thread::sleep(std::time::Duration::from_millis(400));
                println!("🚑 Ambulancia salió del {}", bridge.name);
                return;
            }

            // 2 lanes = faster crossing (no full lock needed)
            println!("➡️➡️ {} cruzando {} (2 carriles)", 
                match vehicle_type {
                    VehicleType::Car => "Auto",
                    VehicleType::SupplyTruck => "Camión",
                    _ => "Vehículo",
                },
                bridge.name
            );
            std::thread::sleep(std::time::Duration::from_millis(600)); // Faster
            println!("✅ Salió del {}", bridge.name);
        }
    }
}

    /// Devuelve una copia del estado actual para que la GUI pueda dibujar.
    pub fn snapshot(&self) -> (usize, usize, Vec<Vehicle>) {
        (self.width, self.height, self.vehicles.clone())
    }

}

