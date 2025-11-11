use rand::{thread_rng, Rng};

/// Lottery: sorteo ponderado por `tickets` (u32).
pub(super) fn pick(lot_ready: Vec<(usize, u32)>) -> Option<usize> {
    if lot_ready.is_empty() {
        return None;
    }
    let total: u64 = lot_ready.iter().map(|&(_, t)| t as u64).sum();
    if total == 0 {
        return None;
    }

    let pick = thread_rng().gen_range(0..total);
    let mut acc: u64 = 0;
    for (idx, tickets) in lot_ready {
        acc += tickets as u64;
        if pick < acc {
            return Some(idx);
        }
    }
    None
}
