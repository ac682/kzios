use erhino_shared::proc::{Pid, ProcessPermission};
use flagset::{flags, FlagSet};

use crate::call::sys_fork;

pub fn fork<P: Into<FlagSet<ProcessPermission>>>(perm: P) -> Result<Pid, ()> {
    let pid = unsafe { sys_fork(perm.into().bits()) };
    if pid < 0 {
        Err(())
    } else {
        Ok(pid as Pid)
    }
}
