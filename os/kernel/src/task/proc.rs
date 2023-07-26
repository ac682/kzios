use alloc::string::String;
use erhino_shared::{
    mem::Address,
    proc::{ExitCode, Pid, ProcessPermission, ProcessState},
};
use flagset::FlagSet;

#[derive(Debug)]
pub enum ProcessSpawnError {

}

pub struct Process {
    pub pid: Pid,
    pub parent: Pid,
    pub name: String,
    pub entry_point: Address,
    pub permissions: FlagSet<ProcessPermission>,
    pub state: ProcessState,
    pub exit_code: ExitCode,
}

impl Process {
    pub fn from_elf(data: &[u8], name: &str) -> Result<Process, ProcessSpawnError> {
        todo!()
    }
}
