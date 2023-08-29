#![no_std]

use core::sync::atomic::{AtomicBool, Ordering};

struct CleanupGuard<'a>(&'a mut Option<Cleanup>);

static CLEANUP_GUARD: AtomicBool = AtomicBool::new(false);

static mut CLEANUP: Option<Cleanup> = None;

impl<'a> CleanupGuard<'a> {
    fn acquire() -> Self {
        loop {
            if CLEANUP_GUARD.compare_exchange_weak(false, true, Ordering::SeqCst, Ordering::Relaxed).is_ok() {
                break;
            }
        }
        CleanupGuard(unsafe { &mut CLEANUP })
    }
}

impl<'a> Drop for CleanupGuard<'a> {
    fn drop(&mut self) {
        CLEANUP_GUARD.store(false, Ordering::SeqCst);
    }
}

struct Cleanup {
}

pub fn init() {
    let cleanup = CleanupGuard::acquire();
    if cleanup.0.is_none() {
        cleanup.0.replace(Cleanup { });
    }
}
