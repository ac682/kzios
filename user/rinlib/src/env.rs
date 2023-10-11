use core::cell::OnceCell;

use erhino_shared::proc::Pid;

pub(crate) static mut PID: OnceCell<Pid> = OnceCell::new();
pub(crate) static mut PARENT_PID: OnceCell<Pid> = OnceCell::new();

pub fn pid() -> Pid {
    unsafe { *PID.get().unwrap() }
}

pub fn parent_pid() -> Pid {
    unsafe { *PARENT_PID.get().unwrap() }
}
