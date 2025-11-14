use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex as StdMutex;

use crate::mypthreads::{my_thread_yield_, MyThreadId};

/// Estructura que representa un mutex cooperativo de nuestra biblioteca.
///
/// La idea es similar a `pthread_mutex_t`, pero implementado en espacio de usuario:
/// - `locked` indica si el recurso está tomado o no.
/// - `owner` guarda el ID del hilo que tiene el mutex (si alguno).
#[derive(Debug)]
pub struct MyMutex {
    /// Indica si el mutex está bloqueado (`true`) o libre (`false`).
    locked: AtomicBool,
    /// ID del hilo que posee el mutex actualmente.
    owner: StdMutex<Option<MyThreadId>>,
}

impl MyMutex {
    /// Crea un nuevo mutex desbloqueado, sin dueño.
    pub fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
            owner: StdMutex::new(None),
        }
    }

    /// Intenta adquirir el mutex bloqueando hasta lograrlo.
    ///
    /// Usa espera **cooperativa**, es decir, mientras el mutex esté ocupado
    /// llama a `my_thread_yield_()` para ceder la CPU a otros hilos.
    ///
    /// `current_tid` es el ID del hilo que intenta tomar el mutex.
    pub fn lock(&self, current_tid: MyThreadId) {
        while self.locked.swap(true, Ordering::Acquire) {
            // Otro hilo tiene el mutex → cedo el procesador cooperativamente.
            my_thread_yield_();
        }

        // En este punto el mutex estaba libre y ya lo marcamos como locked.
        *self.owner.lock().unwrap() = Some(current_tid);
    }

    /// Libera el mutex si el hilo actual es su dueño.
    ///
    /// Devuelve `Ok(())` si se liberó correctamente, o un error si
    /// otro hilo intentó liberar un mutex que no le pertenece.
    pub fn unlock(&self, current_tid: MyThreadId) -> Result<(), &'static str> {
        let mut owner_guard = self.owner.lock().unwrap();

        match *owner_guard {
            Some(owner_id) if owner_id == current_tid => {
                // El hilo actual sí es el dueño → puede liberar.
                *owner_guard = None;
                self.locked.store(false, Ordering::Release);
                Ok(())
            }
            _ => Err("current thread is not the owner of this mutex"),
        }
    }

    /// Intenta adquirir el mutex **sin bloquear**.
    ///
    /// - Si lo logra, devuelve `true` y se convierte en el dueño.
    /// - Si ya estaba bloqueado por otro hilo, devuelve `false` inmediatamente.
    pub fn try_lock(&self, current_tid: MyThreadId) -> bool {
        // Si locked era false, lo pone en true y devuelve false → entramos.
        // Si locked era true, devuelve true → alguien más lo tiene.
        if !self.locked.swap(true, Ordering::Acquire) {
            *self.owner.lock().unwrap() = Some(current_tid);
            true
        } else {
            false
        }
    }

    /// "Destruye" el mutex.
    ///
    /// En esta implementación no libera recursos del sistema operativo,
    /// pero dejamos el mutex en estado limpio (libre y sin dueño).
    pub fn destroy(&self) {
        self.locked.store(false, Ordering::Release);
        *self.owner.lock().unwrap() = None;
    }
}

/// Función de conveniencia que crea un nuevo mutex.
///
/// Esto imita el estilo de `my_mutex_init()` del enunciado, pero en Rust
/// es más natural devolver el objeto ya inicializado.
///
/// ```rust
/// use proyecto1::mypthreads::my_mutex_init;
///
/// let m = my_mutex_init();
/// ```
pub fn my_mutex_init() -> MyMutex {
    MyMutex::new()
}

/// Función de conveniencia que limpia el estado del mutex.
///
/// No libera memoria (porque en Rust eso lo hace el compilador),
/// pero deja el mutex desbloqueado y sin dueño.
pub fn my_mutex_destroy(mutex: &MyMutex) {
    mutex.destroy();
}

/// Envoltorio para `MyMutex::lock`, con una API más cercana al enunciado.
///
/// El código que llame a esta función debe conocer su propio `MyThreadId`.
pub fn my_mutex_lock(mutex: &MyMutex, current_tid: MyThreadId) {
    mutex.lock(current_tid);
}

/// Envoltorio para `MyMutex::unlock`.
pub fn my_mutex_unlock(mutex: &MyMutex, current_tid: MyThreadId) -> Result<(), &'static str> {
    mutex.unlock(current_tid)
}

/// Envoltorio para `MyMutex::try_lock`.
pub fn my_mutex_trylock(mutex: &MyMutex, current_tid: MyThreadId) -> bool {
    mutex.try_lock(current_tid)
}
