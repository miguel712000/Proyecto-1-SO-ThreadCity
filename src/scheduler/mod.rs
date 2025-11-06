use once_cell::sync::Lazy;
use rand::{thread_rng, Rng};
use std::sync::Mutex;

use crate::mypthreads::{with_threads, MyThreadId, SchedulerType, ThreadState};

static RUN_QUEUE: Lazy<Mutex<Vec<usize>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Índice circular para Round Robin (sobre los candidatos READY de tipo RR).
static RR_CURSOR: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

pub fn scheduler_add(tid: usize) {
    let mut rq = RUN_QUEUE.lock().unwrap();
    rq.push(tid);
}

/*
// Por ahora: regresa el primero (round robin falso)
pub fn scheduler_next() -> Option<usize> {
    let mut rq = RUN_QUEUE.lock().unwrap();
    if rq.is_empty() {
        None
    } else {
        // versión tonta: quita y pone al final
        let tid = rq.remove(0);
        rq.push(tid);
        Some(tid)
    }
}
*/

// Selecciona el próximo hilo a ejecutar (devuelve su ID) con prioridad:
// 1) RealTime -> hilo READY con menor `deadline_ms`
// 2) Lottery  -> sorteo ponderado por `tickets`
// 3) RoundRobin -> rotación circular
pub fn scheduler_next() -> Option<MyThreadId> {
    // 1) Construir snapshot de candidatos READY por política bajo lock.
    let (mut rt_ready, lot_ready, rr_ready) = with_threads(|table| {
        let mut rt: Vec<(usize, u64)> = Vec::new(); // (idx, deadline_ms)
        let mut lot: Vec<(usize, u32)> = Vec::new(); // (idx, tickets)
        let mut rr: Vec<usize> = Vec::new(); // idx

        for (idx, t) in table.iter().enumerate() {
            if t.state != ThreadState::Ready {
                continue;
            }
            match t.scheduler_type {
                SchedulerType::RealTime => {
                    if let Some(d) = t.deadline_ms {
                        rt.push((idx, d));
                    } else {
                        // Si no tiene deadline, trátalo como RR.
                        rr.push(idx);
                    }
                }
                SchedulerType::Lottery => {
                    // Asegurar al menos 1 ticket para participar.
                    let tickets = if t.tickets == 0 { 1 } else { t.tickets };
                    lot.push((idx, tickets));
                }
                SchedulerType::RoundRobin => {
                    rr.push(idx);
                }
            }
        }

        (rt, lot, rr)
    });

    // 2) Decisión fuera del lock (operaciones O(n) rápidas).

    // --- Prioridad 1: Tiempo Real (menor deadline_ms) ---
    if !rt_ready.is_empty() {
        // Ordenar por deadline ascendente y tomar el primero.
        rt_ready.sort_by_key(|&(_, d)| d);
        return Some(rt_ready[0].0);
    }

    // --- Prioridad 2: Lottery (sorteo ponderado por tickets) ---
    if !lot_ready.is_empty() {
        let total: u64 = lot_ready.iter().map(|&(_, t)| t as u64).sum();
        if total > 0 {
            let pick = thread_rng().gen_range(0..total);
            let mut acc: u64 = 0;
            for (idx, tickets) in lot_ready {
                acc += tickets as u64;
                if pick < acc {
                    return Some(idx);
                }
            }
        }
        // Si por alguna razón no se escogió (p.ej. total==0), caemos a RR.
    }

    // --- Prioridad 3: Round Robin (rotación circular) ---
    if !rr_ready.is_empty() {
        let mut cursor = RR_CURSOR.lock().expect("RR_CURSOR poisoned");
        if *cursor >= rr_ready.len() {
            *cursor = 0;
        }
        let chosen = rr_ready[*cursor];
        *cursor = (*cursor + 1) % rr_ready.len();
        return Some(chosen);
    }

    // No hay hilos READY.
    None
}
