use proyecto1::mypthreads::{
    my_mutex_init, my_mutex_lock, my_mutex_unlock, my_mutex_trylock,
    my_thread_create, SchedulerType,
};
use proyecto1::scheduler;

fn main() {
    println!("========================");
    println!("  PRUEBAS PROYECTO SO");
    println!("========================\n");

    // Test 1: Mutex básico
    println!("Test 1: Mutex básico (lock/unlock)...");
    if test_mutex_basico() {
        println!("Test 1 OK\n");
    } else {
        println!("Test 1 FALLÓ\n");
    }

    // Test 2: Mutex try_lock
    println!("Test 2: Mutex try_lock...");
    if test_mutex_trylock() {
        println!("Test 2 OK\n");
    } else {
        println!("Test 2 FALLÓ\n");
    }

    // Test 3: Scheduler + creación de hilos
    println!("Test 3: Scheduler Round Robin simple...");
    if test_scheduler_round_robin() {
        println!("Test 3 OK\n");
    } else {
        println!("Test 3 FALLÓ\n");
    }

    println!("Fin de pruebas.");
}

// --------------------
// TEST 1: Mutex básico
// --------------------
fn test_mutex_basico() -> bool {
    // Creamos un mutex
    let m = my_mutex_init();

    // Simulamos que el hilo con ID 0 adquiere el mutex
    my_mutex_lock(&m, 0);
    println!("  [T1] Hilo 0 adquirió el mutex");

    // El mismo hilo lo libera
    match my_mutex_unlock(&m, 0) {
        Ok(()) => {
            println!("  [T1] Hilo 0 liberó el mutex");
            true
        }
        Err(e) => {
            println!("  [T1] Error al liberar mutex: {e}");
            false
        }
    }
}

// --------------------------
// TEST 2: Mutex try_lock
// --------------------------
fn test_mutex_trylock() -> bool {
    let m = my_mutex_init();

    // Hilo 1 intenta tomar el mutex con try_lock
    let ok1 = my_mutex_trylock(&m, 1);
    println!("  [T2] Hilo 1 try_lock → {ok1}");
    if !ok1 {
        println!("  [T2] Hilo 1 debería haber podido tomar el mutex (estaba libre).");
        return false;
    }

    // Mientras el hilo 1 lo tiene, el hilo 2 NO debería poder tomarlo con try_lock
    let ok2 = my_mutex_trylock(&m, 2);
    println!("  [T2] Hilo 2 try_lock (con mutex ya tomado) → {ok2}");
    if ok2 {
        println!("  [T2] Error: hilo 2 no debería haber podido tomar el mutex.");
        return false;
    }

    // Liberamos el mutex desde el hilo 1
    if my_mutex_unlock(&m, 1).is_err() {
        println!("  [T2] Error: hilo 1 no pudo liberar el mutex.");
        return false;
    }

    true
}

// --------------------------------------
// TEST 3: Scheduler + my_thread_create
// --------------------------------------

// Función de ejemplo para los "hilos"
fn dummy() {
    println!("  [dummy] ejecutando función dummy");
}

fn test_scheduler_round_robin() -> bool {
    // Creamos tres hilos con scheduler RoundRobin
    let t1 = my_thread_create(dummy, SchedulerType::RoundRobin)
        .expect("no se pudo crear hilo 1");
    let t2 = my_thread_create(dummy, SchedulerType::RoundRobin)
        .expect("no se pudo crear hilo 2");
    let t3 = my_thread_create(dummy, SchedulerType::RoundRobin)
        .expect("no se pudo crear hilo 3");

    println!("  [T3] Hilos creados con IDs: {t1}, {t2}, {t3}");

    // Ahora probamos que scheduler_next vaya rotando
    let n1 = scheduler::scheduler_next().expect("scheduler vacío en paso 1");
    let n2 = scheduler::scheduler_next().expect("scheduler vacío en paso 2");
    let n3 = scheduler::scheduler_next().expect("scheduler vacío en paso 3");
    let n4 = scheduler::scheduler_next().expect("scheduler vacío en paso 4");

    println!("  [T3] Orden devuelto por scheduler_next(): {n1}, {n2}, {n3}, {n4}");

    // Para un Round Robin sencillo sobre 3 hilos, esperaríamos algo tipo:
    //   t1, t2, t3, t1, ...
    // Asumiendo que los IDs van 0,1,2 en orden de creación, esto debería cumplirse.
    let esperados = [t1, t2, t3, t1];
    let recibidos = [n1, n2, n3, n4];

    let ok = esperados == recibidos;
    if !ok {
        println!("  [T3] Esperado: {:?}, Recibido: {:?}", esperados, recibidos);
    }

    ok
}
