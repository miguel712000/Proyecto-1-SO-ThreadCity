pub mod mutex;
pub mod thread;

pub use mutex::*;
pub use thread::{
    my_thread_chsched, my_thread_create, my_thread_detach, my_thread_end, my_thread_join,
    my_thread_yield_, with_threads, with_threads_mut, MyThreadId, SchedulerType,
    ThreadControlBlock, ThreadState,
};

/* 
pub fn scheduler_next() -> Option<u32> {
    None
}
*/