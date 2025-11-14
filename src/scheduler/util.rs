use once_cell::sync::Lazy;
use std::sync::{Mutex, atomic::{AtomicBool, Ordering}};
use std::time::Instant;

/// Cola que mantenías (aunque hoy no se usa para elegir).
/// Se conserva porque tu `scheduler_add()` la usa.
pub(super) static RUN_QUEUE: Lazy<Mutex<Vec<usize>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

/// Índice circular para Round Robin (sobre los candidatos READY de tipo RR).
pub(super) static RR_CURSOR: Lazy<Mutex<usize>> =
    Lazy::new(|| Mutex::new(0));

// Set del reloj y explosión ----
static START: Lazy<Instant> = Lazy::new(Instant::now);
static EXPLODED: AtomicBool = AtomicBool::new(false);

pub(super) fn push_run_queue(tid: usize) {
    let mut rq = RUN_QUEUE.lock().unwrap();
    rq.push(tid);
}

// tiempo transcurrido desde el inicio (ms)
pub(super) fn now_ms() -> u64 {
    START.elapsed().as_millis() as u64
}

// marcar explosión
pub(super) fn mark_explosion() {
    EXPLODED.store(true, Ordering::SeqCst);
}

// leer estado de explosión (lo exponemos via mod.rs)
pub(super) fn exploded() -> bool {
    EXPLODED.load(Ordering::SeqCst)
}