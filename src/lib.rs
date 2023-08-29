#![no_std]

use core::mem::{size_of};
use core::ptr::{self};
use core::sync::atomic::{AtomicBool, Ordering};
use pc_ints::{int_31h_ax_0100h_rm_alloc, int_31h_ax_0201h_set_rm_int};

struct CleanupGuard(&'static mut Option<Cleanup>);

static CLEANUP_GUARD: AtomicBool = AtomicBool::new(false);

static mut CLEANUP: Option<Cleanup> = None;

impl CleanupGuard {
    fn acquire() -> Self {
        loop {
            if CLEANUP_GUARD.compare_exchange_weak(false, true, Ordering::SeqCst, Ordering::Relaxed).is_ok() {
                break;
            }
        }
        CleanupGuard(unsafe { &mut CLEANUP })
    }
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        CLEANUP_GUARD.store(false, Ordering::SeqCst);
    }
}

struct Cleanup {
    ctrl_c_flag: *mut u8,
}

fn init_raw() -> CleanupGuard {
    let cleanup = CleanupGuard::acquire();
    if cleanup.0.is_none() {
        let handlers_segment = int_31h_ax_0100h_rm_alloc((HANDLERS.len().checked_add(15).unwrap() / 16).try_into().unwrap())
            .expect("cannot allocate real-mode memory for cleanup");
        let handlers_segment = handlers_segment.ax_segment;
        assert!(size_of::<usize>() == size_of::<u32>());
        let handlers_addr = ((handlers_segment as u32) << 4) as usize;
        unsafe { ptr::copy_nonoverlapping(HANDLERS.as_ptr(), handlers_addr as *mut u8, HANDLERS.len()); }
        int_31h_ax_0201h_set_rm_int(0x23, handlers_segment, 0x0000);
        cleanup.0.replace(Cleanup { ctrl_c_flag: handlers_addr as *mut u8 });
    }
    cleanup
}

pub fn init() {
    init_raw();
}

pub fn reset_ctrl_c_flag() -> bool {
    let cleanup = init_raw();
    unsafe { ptr::replace(cleanup.0.as_mut().unwrap().ctrl_c_flag, 0) != 0 }
}

const HANDLERS: &[u8] = &[
    0x00, // ctrl_c_flag (0000h): db 0
    0xC6, 0x06, 0x00, 0x00, 0x01, // ctrl_c_handler (0001h): mov byte [ctrl_c_flag], 1
    0xCF, // iret
];
