use erhino_shared::{mem::Address, proc::Tid};

use crate::call::sys_thread_spawn;

pub enum ThreadSpawnError {
    KernelError,
}

pub struct Thread {
    handle: Tid,
}

impl Thread {
    fn new(handle: Tid) -> Self {
        Self { handle }
    }

    pub fn id(&self) -> Tid{
        self.handle
    }
}

fn thread_wrapper(){

}

pub fn spawn(func: fn()) -> Result<Thread, ThreadSpawnError> {
    unsafe {
        match sys_thread_spawn(thread_wrapper as Address) {
            Ok(tid) => Ok(Thread::new(tid)),
            Err(_) => Err(ThreadSpawnError::KernelError),
        }
    }
}
