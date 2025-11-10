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
};

pub fn scheduler_next() -> Option<u32> {
    None
}
