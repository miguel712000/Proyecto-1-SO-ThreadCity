pub mod mutex;
pub mod thread;


pub use mutex::*;
pub use thread::{
    MyThreadId,
    ThreadState,
    SchedulerType,
    ThreadControlBlock,
    with_threads,
    with_threads_mut,
    my_thread_create,
    my_thread_join,
    my_thread_detach,
    my_thread_chsched,
    my_thread_yield_,
    my_thread_end,
    my_thread_set_tickets,
    my_thread_set_deadline_ms,
    my_thread_id,
    set_current_thread_id
    
};

/* 
pub fn scheduler_next() -> Option<u32> {
    None
}
*/