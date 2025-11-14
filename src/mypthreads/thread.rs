use std::sync::{Mutex, Arc};
//use std::time::{SystemTime, UNIX_EPOCH};  //Importa tipos del módulo estándar de tiempo en Rust.
use crate::scheduler;
use once_cell::sync::Lazy;

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
pub struct ThreadControlBlock {
    /// ID único del hilo dentro de la tabla.
    pub id: MyThreadId,
    /// Estado actual del hilo.
    pub state: ThreadState,
    /// Si algún otro hilo está esperando a que este termine (`join`), aquí se guarda su ID.
    pub waiting_thread_id: Option<MyThreadId>,
    /// Qué scheduler se debe usar para este hilo.
    pub scheduler_type: SchedulerType,
    /// Si es `true`, el hilo no se puede esperar (`join`).
    pub detached: bool,
    /// Función que este hilo debe ejecutar cuando se le asigne CPU.
    pub start_routine: Option<Arc<dyn Fn() + Send + Sync>>,
    // TODO: contexto real más adelante

    // Metadatos de scheduling
    pub tickets: u32, // para Lottery (>=1)
    pub deadline_ms: Option<u64>, // para RT (epoch ms); None si no aplica

                      // TODO: contexto real (ucontext_t equivalente)
}

// =========================
// "Tabla" global de hilos
// =========================

/// Máximo de hilos que vamos a soportar
const MAX_THREADS: usize = 64;

/// Tabla global donde se guardan **todos** los hilos creados.
/// Se protege con `Mutex` porque todos los hilos van a tocarla.
static THREAD_TABLE: Lazy<Mutex<Vec<ThreadControlBlock>>> =
    Lazy::new(|| Mutex::new(Vec::with_capacity(MAX_THREADS)));

/// ID del hilo que está actualmente en ejecución.
/// Si es `None` significa que no hay hilo corriendo (por ejemplo, ya terminaron todos).
static CURRENT_THREAD_ID: Lazy<Mutex<Option<MyThreadId>>> = Lazy::new(|| Mutex::new(None));

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
pub fn my_thread_create<F>(
    start_routine: F, // puede ser fn o closure
    scheduler_type: SchedulerType,
) -> Result<MyThreadId, &'static str>
where 
    F: Fn() + Send + Sync + 'static,
{

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
        start_routine: Some(Arc::new(start_routine)),


        // Defaults de las propiedades para schedule
        tickets: 1,
        deadline_ms: None,
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

// Acceso controlado a la tabla global de hilos (`THREAD_TABLE`).
// Estas funciones encapsulan el uso del `Mutexegúrate que existan estos campos (además de los otros que ya ten` que protege la tabla,
// permitiendo ejecutar un cierre (`closure`) con acceso seguro a los TCB:

/// Otorga **solo lectura** (`&[ThreadControlBlock]`).
pub fn with_threads<F, R>(f: F) -> R
where
    F: FnOnce(&Vec<ThreadControlBlock>) -> R,
{
    let table = THREAD_TABLE.lock().unwrap();
    f(&*table)
}

/// Ejecuta una función con acceso mutable a la tabla de hilos.
///
/// Útil para pruebas o para ajustar metadatos de scheduling (tickets, deadlines, etc.).
pub fn with_threads_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut Vec<ThreadControlBlock>) -> R,
{
    let mut table = THREAD_TABLE.lock().unwrap();
    f(&mut *table)
}

/// Ajusta la cantidad de tickets para Lottery del hilo `tid`.
pub fn my_thread_set_tickets(tid: MyThreadId, tickets: u32) -> Result<(), &'static str> {
    if tickets == 0 {
        return Err("tickets must be >= 1");
    }
    let mut table = THREAD_TABLE.lock().unwrap();
    if tid >= table.len() {
        return Err("thread does not exist");
    }
    table[tid].tickets = tickets;
    Ok(())
}

/// Ajusta el deadline (en ms desde epoch) para RT del hilo `tid`.
/// Usa `None` para limpiar/eliminar el deadline.
pub fn my_thread_set_deadline_ms(
    tid: MyThreadId,
    deadline_ms: Option<u64>,
) -> Result<(), &'static str> {
    let mut table = THREAD_TABLE.lock().unwrap();
    if tid >= table.len() {
        return Err("thread does not exist");
    }
    table[tid].deadline_ms = deadline_ms;
    Ok(())
}

/// Ejecuta la función asociada al hilo `tid`, si existe.
/// 
/// Por ahora esto ejecuta la función de forma síncrona (sin cambio de contexto real),
/// lo cual es suficiente para una simulación cooperativa básica.
pub fn my_thread_run_once(tid: MyThreadId) {
    let maybe_func = {
        let table = THREAD_TABLE.lock().unwrap();
        if let Some(tcb) = table.get(tid) {
            tcb.start_routine.clone()
        } else {
            None
        }
    };

    if let Some(f) = maybe_func {
        // Aquí podríamos marcar RUNNING antes, FINISHED después, etc.
        f();
    }
}

     pub fn my_thread_id() -> MyThreadId {
         CURRENT_THREAD_ID.lock().unwrap().expect("No current thread is running")
     }

     /// Sets the ID of the currently running thread.
/// This is used by the scheduler/main loop to update the global state.
pub fn set_current_thread_id(tid: MyThreadId) {
    *CURRENT_THREAD_ID.lock().unwrap() = Some(tid);
}
