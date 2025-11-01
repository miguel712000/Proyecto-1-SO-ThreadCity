use std::sync::Mutex;
use once_cell::sync::Lazy;

static RUN_QUEUE: Lazy<Mutex<Vec<usize>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn scheduler_add(tid: usize) {
    let mut rq = RUN_QUEUE.lock().unwrap();
    rq.push(tid);
}

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
