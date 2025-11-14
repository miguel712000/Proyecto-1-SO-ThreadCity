/// Tiempo Real: elegir READY con menor `deadline_ms`.
pub(super) fn pick(rt_ready: &mut Vec<(usize, u64)>) -> Option<usize> {
    if rt_ready.is_empty() {
        return None;
    }
    // Ordenar por deadline ascendente y tomar el primero (igual a tu código).
    rt_ready.sort_by_key(|&(_, d)| d);
    Some(rt_ready[0].0)
}
