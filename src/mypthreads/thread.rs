use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::scheduler;

// =========================
// Tipos básicos y estados
// =========================

/// Identificador lógico de un hilo dentro de nuestra biblioteca.
/// Es simplemente el índice dentro de la tabla global de hilos.
pub type MyThreadId = usize;

/// Estado en el que puede estar un hilo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    Ready,
    Running,
    Blocked,
    Finished,
}

/// Tipos de scheduler soportados por la biblioteca.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerType {
    RoundRobin,
    Lottery,
    RealTime,
}

/// Estructura que representa a **un hilo** dentro de la biblioteca.
#[derive(Debug)]
pub struct ThreadControlBlock {
    /// ID único del hilo dentro de la tabla.
    pub id: MyThreadId,
    /// Estado actual del hilo.
    pub state: ThreadState,
    /// Si algún otro hilo está esperando a que este termine (`join`), aquí se guarda su ID.
    pub waiting_thread_id: Option<MyThreadId>,
    /// Qué scheduler se debe usar para este hilo.
    pub scheduler_type: SchedulerType,
    /// Si es `true`, el hilo no se puede esperar (`join`), como en `pthread_detach`.
    pub detached: bool,
    // TODO: contexto real (ucontext_t equivalente)
}

// =========================
// "Tabla" global de hilos
// =========================

/// Máximo de hilos que vamos a soportar
const MAX_THREADS: usize = 64;

/// Tabla global donde se guardan **todos** los hilos creados.
/// Se protege con `Mutex` porque todos los hilos van a tocarla.
static THREAD_TABLE: Lazy<Mutex<Vec<ThreadControlBlock>>> = Lazy::new(|| {
    Mutex::new(Vec::with_capacity(MAX_THREADS))
});

/// ID del hilo que está actualmente en ejecución.
/// Si es `None` significa que no hay hilo corriendo (por ejemplo, ya terminaron todos).
static CURRENT_THREAD_ID: Lazy<Mutex<Option<MyThreadId>>> = Lazy::new(|| {
    Mutex::new(None)
});

// =========================
// Funciones de hilos
// =========================

/// Crea un nuevo hilo dentro de la biblioteca.
///
/// - `start_routine`: función que debería ejecutar el hilo (por ahora solo la guardamos).
/// - `scheduler_type`: con qué scheduler se va a planificar este hilo.
///
/// Devuelve el ID del hilo creado o un error si se alcanzó el máximo.
/// 
/// ```rust
/// use proyecto1::mypthreads::{my_thread_create, SchedulerType};
///
/// fn worker() {
///     println!("hola desde el hilo!");
/// }
///
/// fn main() {
///     let _ = my_thread_create(worker, SchedulerType::RoundRobin);
/// }
/// ```
pub fn my_thread_create(
    _start_routine: fn(),            // por ahora función sin args
    scheduler_type: SchedulerType,
) -> Result<MyThreadId, &'static str> {
    let mut table = THREAD_TABLE.lock().unwrap();

    if table.len() >= MAX_THREADS {
        return Err("max threads reached");
    }

    let id = table.len();

    let tcb = ThreadControlBlock {
        id,
        state: ThreadState::Ready,
        waiting_thread_id: None,
        scheduler_type,
        detached: false,
    };

    // Guardamos el hilo en la tabla global
    table.push(tcb);

    // registrar en scheduler
    scheduler::scheduler_add(id);

    Ok(id)
}

/// Marca el hilo actual como **terminado** y permite que el scheduler
/// escoja el siguiente. También despierta a un hilo que estuviera haciendo `join`
/// sobre este.
pub fn my_thread_end() {
    let mut table = THREAD_TABLE.lock().unwrap();
    let mut current_id_lock = CURRENT_THREAD_ID.lock().unwrap();

    // Si no hay hilo actual, no hay nada que hacer
    let Some(current_id) = *current_id_lock else {
        return;
    };

    // lo vamos a usar después
    let waiter_id_opt: Option<MyThreadId>;

    {
        // scope para no tener 2 préstamos mutables a la vez
        let current = &mut table[current_id];
        current.state = ThreadState::Finished;
        // si alguien estaba esperando este hilo, lo anotamos
        waiter_id_opt = current.waiting_thread_id;
        current.waiting_thread_id = None;
    }

    // si había alguien esperando, lo pasamos a READY
    if let Some(waiter_id) = waiter_id_opt {
        if let Some(waiter) = table.get_mut(waiter_id) {
            waiter.state = ThreadState::Ready;
        }
    }

    // pedir el siguiente hilo al scheduler
    if let Some(next_id) = scheduler::scheduler_next() {
        if let Some(next) = table.get_mut(next_id) {
            next.state = ThreadState::Running;
        }
        *current_id_lock = Some(next_id);
    } else {
        *current_id_lock = None;
    }
}

/// Cede voluntariamente el procesador a otro hilo según el scheduler.
pub fn my_thread_yield_() {
    let mut table = THREAD_TABLE.lock().unwrap();
    let mut current_id_lock = CURRENT_THREAD_ID.lock().unwrap();

    // si no hay hilo actual, nada que hacer
    let Some(current_id) = *current_id_lock else {
        return;
    };

    // pedir otro hilo al scheduler
    let Some(next_id) = scheduler::scheduler_next() else {
        return;
    };

    // si el scheduler devolvió el mismo hilo, no hacemos nada
    if next_id == current_id {
        return;
    }

    // actual pasa a READY
    if let Some(current) = table.get_mut(current_id) {
        current.state = ThreadState::Ready;
    }

    // el otro pasa a RUNNING
    if let Some(next) = table.get_mut(next_id) {
        next.state = ThreadState::Running;
    }

    // actualizamos el hilo actual
    *current_id_lock = Some(next_id);
    // TODO: aquí iría el cambio de contexto real
}

/// Bloquea el hilo actual hasta que el hilo con ID `target_id` termine.
///
/// Si el hilo ya había terminado, devuelve `Ok(())` inmediato.
///
/// Importante: como todavía no tenemos cambio de contexto real,
/// esta versión usa un loop con `my_thread_yield_()`.
pub fn my_thread_join(target_id: MyThreadId) -> Result<(), &'static str> {
    let mut table = THREAD_TABLE.lock().unwrap();
    let current_id_lock = CURRENT_THREAD_ID.lock().unwrap();

    // validar que el hilo exista
    if target_id >= table.len() {
        return Err("thread does not exist");
    }

    // si ya terminó, nada que esperar
    if table[target_id].state == ThreadState::Finished {
        return Ok(());
    }

    // quién soy yo?
    let Some(current_id) = *current_id_lock else {
        return Err("no current thread");
    };

    // marco que el target me despierte cuando termine
    table[target_id].waiting_thread_id = Some(current_id);

    // bloquearme yo
    if let Some(me) = table.get_mut(current_id) {
        me.state = ThreadState::Blocked;
    }

    // soltar locks ANTES del loop
    drop(table);
    drop(current_id_lock);

    // esperar cooperativamente
    loop {
        {
            let table = THREAD_TABLE.lock().unwrap();
            if table[target_id].state == ThreadState::Finished {
                break;
            }
        }
        my_thread_yield_();
    }

    Ok(())
}

/// Marca un hilo como "detached", es decir, que no va a ser `join`eado.
pub fn my_thread_detach(tid: MyThreadId) -> Result<(), &'static str> {
    let mut table = THREAD_TABLE.lock().unwrap();
    if tid >= table.len() {
        return Err("thread does not exist");
    }
    table[tid].detached = true;
    Ok(())
}

/// Cambia el scheduler asignado a un hilo en tiempo de ejecución.
pub fn my_thread_chsched(tid: MyThreadId, new_sched: SchedulerType) -> Result<(), &'static str> {
    let mut table = THREAD_TABLE.lock().unwrap();
    if tid >= table.len() {
        return Err("thread does not exist");
    }
    table[tid].scheduler_type = new_sched;
    Ok(())
}
