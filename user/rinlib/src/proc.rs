use erhino_shared::proc::{ExitCode, Pid, ProcessPermission};
use flagset::FlagSet;

use crate::call::{sys_exit, sys_fork};

pub fn exit(code: ExitCode) -> ! {
    unsafe {
        loop {
            sys_exit(code)
        }
    };
}

pub fn fork<P: Into<FlagSet<ProcessPermission>>>(perm: P) -> Result<Pid, ()> {
    let pid = unsafe { sys_fork(perm.into().bits()) };
    if pid < 0 {
        Err(())
    } else {
        Ok(pid as Pid)
    }
}
