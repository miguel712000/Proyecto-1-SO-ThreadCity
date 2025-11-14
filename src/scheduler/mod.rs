use crate::mypthreads::{with_threads, MyThreadId, SchedulerType, ThreadState};

mod util;
mod rt;
mod lottery;
mod rr;

/// Registrar un hilo en la cola.
/// Push al RUN_QUEUE.
pub fn scheduler_add(tid: usize) {
    util::push_run_queue(tid);
}

/// Permite consultar desde fuera si ya "explotó" la planta.
pub fn plant_exploded() -> bool {
    util::exploded()
}

/// Revisa si algún hilo RT READY ya venció su deadline y marca "explosión".
fn sweep_deadlines_and_flag() {
    let now = util::now_ms();
    // Sólo marcamos el flag; no cambiamos estados.
    let miss = with_threads(|table| {
        table.iter().any(|t|
            t.state != ThreadState::Finished
            && t.scheduler_type == SchedulerType::RealTime
            && t.deadline_ms.is_some()
            && t.deadline_ms.unwrap() <= now
        )
    });
    if miss { util::mark_explosion(); }
}

/// Selecciona el próximo hilo a ejecutar (devuelve su ID) con prioridad:
/// 1) RealTime  -> hilo READY con menor `deadline_ms`
/// 2) Lottery   -> sorteo ponderado por `tickets`
/// 3) RoundRobin-> rotación circular
pub fn scheduler_next() -> Option<MyThreadId> {

    // Barrido de deadlines antes de decidir
    sweep_deadlines_and_flag();
    // 1) Snapshot de candidatos READY bajo lock (misma lógica que tenías).
    let (mut rt_ready, lot_ready, rr_ready) = with_threads(|table| {
        let mut rt: Vec<(usize, u64)> = Vec::new(); // (idx, deadline_ms)
        let mut lot: Vec<(usize, u32)> = Vec::new(); // (idx, tickets)
        let mut rr: Vec<usize> = Vec::new();         // idx

        for (idx, t) in table.iter().enumerate() {
            if t.state != ThreadState::Ready {
                continue;
            }
            match t.scheduler_type {
                SchedulerType::RealTime => {
                    if let Some(d) = t.deadline_ms {
                        rt.push((idx, d));
                    } else {
                        // Si no tiene deadline, trátalo como RR (igual que tu código).
                        rr.push(idx);
                    }
                }
                SchedulerType::Lottery => {
                    // Mantengo tu regla: si tickets == 0, usar 1.
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

    // 2) Decisión fuera del lock (idéntico a tu flujo):

    // --- Prioridad 1: Tiempo Real (menor deadline_ms) ---
    if let Some(tid) = rt::pick(&mut rt_ready) {
        return Some(tid);
    }

    // --- Prioridad 2: Lottery (sorteo ponderado por tickets) ---
    if let Some(tid) = lottery::pick(lot_ready) {
        return Some(tid);
    }

    // --- Prioridad 3: Round Robin (rotación circular) ---
    rr::pick(rr_ready)
}
