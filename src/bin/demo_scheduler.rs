use std::collections::HashMap;
use std::sync::Arc;

use proyecto1::mypthreads::{
    my_thread_create,
    with_threads,
    with_threads_mut,
    MyThreadId,
    SchedulerType,
    ThreadState,
};
use proyecto1::scheduler;

fn main() {
    println!("========================================");
    println!("   DEMO SCHEDULER: RT + LOTTERY + RR");
    println!("========================================\n");

    // 1) Crear hilos de distintos tipos
    let rt1 = my_thread_create(hilo_rt, SchedulerType::RealTime).unwrap();
    let lot1 = my_thread_create(hilo_lottery1, SchedulerType::Lottery).unwrap();
    let lot2 = my_thread_create(hilo_lottery2, SchedulerType::Lottery).unwrap();
    let rr1  = my_thread_create(hilo_rr1, SchedulerType::RoundRobin).unwrap();
    let rr2  = my_thread_create(hilo_rr2, SchedulerType::RoundRobin).unwrap();

    // 2) Ajustar metadatos: deadlines y tickets
    with_threads_mut(|table| {
        // rt1: deadline más urgente
        table[rt1].deadline_ms = Some(1000);

        // lot1 y lot2: distintos tickets
        table[lot1].tickets = 1;  // baja prioridad en Lottery
        table[lot2].tickets = 5;  // más probabilidad

        // Los RR no necesitan campos extra para este demo
    });

    println!("Hilos creados:");
    with_threads(|table| {
        for t in table {
            println!(
                "  tid={} type={:?} state={:?} tickets={} deadline={:?}",
                t.id, t.scheduler_type, t.state, t.tickets, t.deadline_ms
            );
        }
    });

    println!("\n----------------------------------------");
    println!("FASE 1: Hay un hilo RealTime READY");
    println!("----------------------------------------");

    // Mientras rt1 esté READY, el scheduler siempre debería elegirlo.
    for i in 0..5 {
        let next = scheduler::scheduler_next();
        match next {
            Some(tid) => {
                with_threads(|table| {
                    let t = &table[tid];
                    /*println!(
                        "  iter {i}: scheduler_next() → tid={} ({:?})",
                        tid, t.scheduler_type
                    );*/
                    if let Some(f) = &t.start_routine {

                    // Clonamos el Arc para poder llamar sin moverlo
                    let func = Arc::clone(f);
                    (func)();
                }
                });
            }
            None => println!("  iter {i}: scheduler_next() → None (no hay hilos READY)"),
        }
    }


    // Ahora "simulamos" que el hilo RT terminó o está bloqueado.
    with_threads_mut(|table| {
        table[rt1].state = ThreadState::Blocked;
    });

    println!("\n----------------------------------------");
    println!("FASE 2: Sin RT, con Lottery + RR");
    println!("----------------------------------------");

    // Vamos a llamar muchas veces a scheduler_next para ver la tendencia Lottery.
    let mut counts: HashMap<MyThreadId, u32> = HashMap::new();
    
    for i in 0..50 {
        if let Some(tid) = scheduler::scheduler_next() {
            // Contador de cuántas veces salió este tid
            *counts.entry(tid).or_insert(0) += 1;

            // Miramos la info del hilo y mostramos detalles
            with_threads(|table| {
                let t = &table[tid];
                /*println!(
                    "  iter {:02}: scheduler_next() → tid={} type={:?} tickets={} deadline={:?}",
                    i,
                    tid,
                    t.scheduler_type,
                    t.tickets,
                    t.deadline_ms
                );*/

                // Si querés, ejecutamos la función dummy asociada a ese hilo
                if let Some(f) = &t.start_routine {

                    // Clonamos el Arc para poder llamar sin moverlo
                    let func = Arc::clone(f);
                    (func)();
                }
            });
        } else {
            println!("  iter {:02}: scheduler_next() → None (no hay hilos READY)", i);
        }
    }


    println!("Veces que salió elegido cada tid (50 iteraciones):");
    for (tid, c) in counts.iter() {
        println!("  tid {tid}: {c} veces");
    }

    println!("\nSegún los tickets, esperamos que:");
    println!("  - lot2 (más tickets) salga más veces que lot1.");
    println!("  - hilos RR casi no aparezcan (Lottery tiene prioridad sobre RR).");

    // Bloquear o marcar como Finished los hilos de Lottery
    with_threads_mut(|table| {
        // ojo: usá los tid correctos de lot1 y lot2
        table[lot1].state = ThreadState::Blocked;
        table[lot2].state = ThreadState::Blocked;
    });

    println!("\n----------------------------------------");
    println!("FASE 3: Sin RT ni Lottery, SOLO Round Robin");
    println!("----------------------------------------");

    //Ahora el scheduler solo tiene RR en READY
    for i in 0..10 {
        if let Some(tid) = scheduler::scheduler_next() {
            with_threads(|table| {
                let t = &table[tid];
                /*println!(
                    "  iter {:02}: scheduler_next() → tid={} type={:?}",
                    i, tid, t.scheduler_type
                );*/
                if let Some(f) = &t.start_routine {

                    // Clonamos el Arc para poder llamar sin moverlo
                    let func = Arc::clone(f);
                    (func)();
                }

            });
        } else {
            println!("  iter {:02}: scheduler_next() → None (no hay hilos READY)", i);
        }
    }

    println!("\nDemo terminada.");
}

// ---------------------------
// Funciones dummy de los hilos
// ---------------------------

fn hilo_rt() {
    println!("[RT] Ejecutando hilo de tiempo real");
}

fn hilo_lottery1() {
    println!("[LOTTERY-1] Ejecutando hilo lottery con pocos tickets");
}

fn hilo_lottery2() {
    println!("[LOTTERY-2] Ejecutando hilo lottery con muchos tickets");
}

fn hilo_rr1() {
    println!("[RR-1] Ejecutando hilo Round Robin");
}

fn hilo_rr2() {
    println!("[RR-2] Ejecutando hilo Round Robin");
}
