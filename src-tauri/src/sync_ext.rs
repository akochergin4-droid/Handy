//! Poison-tolerant locking.
//!
//! A `std::sync::Mutex` is poisoned when a thread panics while holding it, and
//! every later `lock().unwrap()` then panics too, so one failure in a worker
//! thread takes down the recorder, the model manager, and anything else that
//! touches the same state. These helpers log the poisoning and carry on with
//! the data as the panicking thread left it.

use std::sync::{Condvar, Mutex, MutexGuard};

pub trait LockExt<T> {
    /// Lock the mutex, recovering the guard if a previous holder panicked.
    fn lock_or_recover(&self) -> MutexGuard<'_, T>;
}

impl<T> LockExt<T> for Mutex<T> {
    fn lock_or_recover(&self) -> MutexGuard<'_, T> {
        self.lock().unwrap_or_else(|poisoned| {
            log::error!("mutex poisoned by a panicking thread; recovering its state");
            poisoned.into_inner()
        })
    }
}

pub trait CondvarExt {
    /// `Condvar::wait` that recovers the guard if the mutex was poisoned.
    fn wait_or_recover<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T>;
}

impl CondvarExt for Condvar {
    fn wait_or_recover<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        self.wait(guard).unwrap_or_else(|poisoned| {
            log::error!("mutex poisoned by a panicking thread; recovering its state");
            poisoned.into_inner()
        })
    }
}
