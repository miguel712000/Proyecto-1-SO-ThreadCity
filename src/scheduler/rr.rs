use super::util::RR_CURSOR;

/// Round Robin: rotación justa con cursor circular.
pub(super) fn pick(rr_ready: Vec<usize>) -> Option<usize> {
    if rr_ready.is_empty() {
        return None;
    }
    let mut cursor = RR_CURSOR.lock().expect("RR_CURSOR poisoned");
    if *cursor >= rr_ready.len() {
        *cursor = 0;
    }
    let chosen = rr_ready[*cursor];
    *cursor = (*cursor + 1) % rr_ready.len();
    Some(chosen)
}
