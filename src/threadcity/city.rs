// city.rs - tiny city model with very simple movement logic
use std::sync::{Arc, Mutex};
use crate::threadcity::entities::{Vehicle, VehicleType, Bridge};

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
        Bridge { id: 1, name: String::from("Puente Norte"), mutex: Arc::new(Mutex::new(())) },
        Bridge { id: 2, name: String::from("Puente Central"), mutex: Arc::new(Mutex::new(())) },
        Bridge { id: 3, name: String::from("Puente Sur"), mutex: Arc::new(Mutex::new(())) },
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
    pub fn step(&mut self) -> bool {
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
        self.cross_bridge(vtype, bridge_id);
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

    pub fn cross_bridge(&self, vehicle_type: VehicleType, bridge_id: usize) {
        let bridge = &self.bridges[bridge_id - 1]; // los puentes empiezan en 1
        println!("{} quiere cruzar el {}", 
            match vehicle_type {
                VehicleType::Ambulance => "🚑 Ambulancia",
                VehicleType::Car => "🚗 Auto",
                VehicleType::Boat => "🛥️ Barco",
                VehicleType::SupplyTruck => "🚚 Camión",
            },
            bridge.name
        );

        // Si es ambulancia, cruza primero (bloquea inmediatamente)
        if vehicle_type == VehicleType::Ambulance {
            let _lock = bridge.mutex.lock().unwrap();
            println!("🚑 Ambulancia cruzando el {}", bridge.name);
            std::thread::sleep(std::time::Duration::from_millis(800));
            println!("🚑 Ambulancia salió del {}", bridge.name);
            return;
        }

        // Otros vehículos esperan el turno
        let _lock = bridge.mutex.lock().unwrap();
        println!("{} cruzando el {}", 
            match vehicle_type {
                VehicleType::Car => "🚗 Auto",
                VehicleType::Boat => "🛥️ Barco",
                VehicleType::SupplyTruck => "🚚 Camión",
                VehicleType::Ambulance => "🚑 Ambulancia", // redundante pero seguro
            },
            bridge.name
        );
        std::thread::sleep(std::time::Duration::from_millis(1000));
        println!("{} salió del {}", 
            match vehicle_type {
                VehicleType::Car => "🚗 Auto",
                VehicleType::Boat => "🛥️ Barco",
                VehicleType::SupplyTruck => "🚚 Camión",
                VehicleType::Ambulance => "🚑 Ambulancia",
            },
            bridge.name
        );
    }

}

