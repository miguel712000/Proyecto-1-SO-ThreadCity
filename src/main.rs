mod gui; // 

use proyecto1::threadcity::city::City;
use proyecto1::threadcity::entities::VehicleType;
use proyecto1::mypthreads::{
    my_thread_create,
    with_threads,
    SchedulerType,
    my_thread_id,
    set_current_thread_id,
};
use proyecto1::scheduler;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread::{self, sleep};
use crate::gui::{run_gui, SharedCity};

fn run_simulation(city: SharedCity) {

    // === Estado inicial ===
    {
        let c = city.lock().unwrap();
        println!("=== Estado inicial de la ciudad ===");
        c.print_state();
        println!("===================================\n");
    }

    // Crear vehículos
    {
        let mut c = city.lock().unwrap();
        c.spawn_vehicle((0,0), (4,4), VehicleType::Car);
        c.spawn_vehicle((4,0), (0,4), VehicleType::Ambulance);
        c.spawn_vehicle((0,2), (4,2), VehicleType::Boat);
    }

    // ================================
    // CREAR HILO DEL AUTO
    // ================================
    let city_clone = Arc::clone(&city);
    let _car_thread = my_thread_create(
        move || {
            for step_count in 0..20 {

                let tid = my_thread_id();
                {
                    let mut c = city_clone.lock().unwrap();

                    // Pequeña lógica de puente
                    if step_count == 5 {
                        c.cross_bridge(VehicleType::Car, 1, tid);
                    }

                    let done = c.step(tid);
                    c.print_state();

                    if done {
                        println!("🚗 El Auto llegó a su destino ✅");
                        break;
                    }
                }

                std::thread::sleep(Duration::from_millis(300));
            }
        },
        SchedulerType::RoundRobin,
    ).unwrap();

    // ================================
    // BUCLE PRINCIPAL DEL SCHEDULER
    // ================================
    loop {
        let mut all_done = false;

        if let Some(tid) = scheduler::scheduler_next() {

            set_current_thread_id(tid);

            with_threads(|table| {
                let t = &table[tid];
                if let Some(f) = &t.start_routine {
                    f();
                }
            });

            // Revisar si terminó
            let mut c = city.lock().unwrap();
            all_done = c.step(tid);
        }

        if all_done {
            println!("🏁 Todos los vehículos llegaron, fin de la simulación.");
            break;
        }

        sleep(Duration::from_millis(100));
    }

    println!("Simulation finished.");
}

fn main() {

    // Crear ciudad compartida
    let city: SharedCity = Arc::new(Mutex::new(City::new(5, 5)));

    // Lanzar la simulación
    let city_for_sim = city.clone();
    thread::spawn(move || {
        run_simulation(city_for_sim);
    });

    // Lanzar la GUI GTK
    run_gui(city);
}
