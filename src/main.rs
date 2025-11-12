use proyecto1::threadcity::city::City;
use proyecto1::threadcity::entities::VehicleType;
use proyecto1::mypthreads::{
    my_thread_create,
    with_threads,
    SchedulerType,
};
use proyecto1::scheduler;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread::sleep;


fn main() {
    // Crear ciudad compartida dentro de Arc<Mutex<...>>
    let city = Arc::new(Mutex::new(City::new(5, 5))); // 5x5 grid, very small
    {
        let c = city.lock().unwrap();
        println!("=== Estado inicial de la ciudad ===");
        c.print_state();
        println!("===================================\n");
    }

    // Crea carros
    {
        let mut c = city.lock().unwrap();
        c.spawn_vehicle((0,0), (4,4), VehicleType::Car);
        c.spawn_vehicle((4,0), (0,4), VehicleType::Ambulance);
        c.spawn_vehicle((0,2), (4,2), VehicleType::Boat);
    }

    


    // 🚗 Crear un hilo simulado del tipo "Auto" usando mypthreads
let city_clone = Arc::clone(&city);
let _car_thread = my_thread_create(
    move || {
        for step_count in 0..20 {
            {
                let mut c = city_clone.lock().unwrap();

                // Pequeña lógica de puente
                if step_count == 5 {
                    c.cross_bridge(VehicleType::Car, 1);
                }

                let done = c.step();
                c.print_state();

                if done {
                    println!("🚗 El Auto llegó a su destino ✅");
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(300));
        }
    },
    SchedulerType::RoundRobin, // ← tipo de planificación
).unwrap();


// 🧠 Bucle principal del planificador
loop {
    let mut all_done = false;

    if let Some(tid) = scheduler::scheduler_next() {
        with_threads(|table| {
            let t = &table[tid];
            if let Some(f) = &t.start_routine {
                f(); // ejecuta el hilo elegido (el Auto)
            }
        });

        // Verifica si ya todos los vehículos llegaron
        let mut c = city.lock().unwrap();
        all_done = c.step(); // si step() devuelve true, todos llegaron
    }

    if all_done {
        println!("🏁 Todos los vehículos llegaron, fin de la simulación.");
        break;
    }

    sleep(Duration::from_millis(100));
}

    println!("Simulation finished.");
}
