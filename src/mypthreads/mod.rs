pub mod thread;
pub mod mutex;

pub use thread::*;
pub use mutex::*;

pub fn scheduler_next() -> Option<u32> {
    
    None
}
