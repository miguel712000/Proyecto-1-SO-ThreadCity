pub mod mutex;
pub mod thread;

pub use mutex::*;
pub use thread::*;

pub fn scheduler_next() -> Option<u32> {
    None
}
