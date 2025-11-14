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

/// Obtener el tiempo actual en milisegundos
/// CORREGIDO: Envoltorio público para now_ms
pub fn now_ms() -> u64 {
    util::now_ms()
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
    
    // 1) Snapshot de candidatos READY bajo lock
    let (mut rt_ready, lot_ready, rr_ready) = with_threads(|table| {
        let mut rt: Vec<(usize, u64)> = Vec::new();
        let mut lot: Vec<(usize, u32)> = Vec::new();
        let mut rr: Vec<usize> = Vec::new();

        for (idx, t) in table.iter().enumerate() {
            if t.state != ThreadState::Ready {
                continue;
            }
            match t.scheduler_type {
                SchedulerType::RealTime => {
                    if let Some(d) = t.deadline_ms {
                        rt.push((idx, d));
                    } else {
                        rr.push(idx);
                    }
                }
                SchedulerType::Lottery => {
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

    // 2) Decisión fuera del lock
    if let Some(tid) = rt::pick(&mut rt_ready) {
        return Some(tid);
    }

    if let Some(tid) = lottery::pick(lot_ready) {
        return Some(tid);
    }

    rr::pick(rr_ready)
}

// ELIMINA esta línea - no necesitas re-exportar
// pub use util::{now_ms, plant_exploded};