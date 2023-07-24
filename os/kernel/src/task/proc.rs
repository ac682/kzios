use alloc::string::String;
use erhino_shared::{proc::{Pid, ProcessPermission, ProcessState, ExitCode}, mem::Address};
use flagset::FlagSet;

pub enum ProcessSpawnError{

}

pub struct Process {
    name: String,
    pid: Pid,
    parent: Pid,
    entry_point: Address,
    permissions: FlagSet<ProcessPermission>,
    state: ProcessState,
    exit_code: ExitCode
}

impl Process{
    pub fn from_elf(data: &[u8], name: &str) -> Result<Process, ProcessSpawnError>{
        todo!()
    }
}