use core::ffi::CStr;

use alloc::{borrow::ToOwned, ffi::CString, string::String};
use erhino_shared::proc::{ExitCode, Pid, ProcessInfo, ProcessPermission};
use flagset::FlagSet;

use crate::call::{sys_exit, sys_fork, sys_inspect, sys_inspect_myself};

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

pub fn inspect(pid: Pid) -> Result<ProcessInfo, ()> {
    let mut info = ProcessInfo {
        name: String::new(),
        pid: 0,
        parent: 0,
        state: erhino_shared::proc::ProcessState::Ready,
        permissions: ProcessPermission::Valid.into(),
    };
    let mut name_buffer = [0u8; 256];
    let ret = unsafe { sys_inspect(pid, &mut info, &mut name_buffer) };
    if ret {
        let mut len = 0usize;
        for i in 0..256 {
            if name_buffer[i] == 0 {
                len = i;
                break;
            }
        }
        if len > 0 {
            let name = String::from_utf8(name_buffer[..len].to_vec());
            info.name = name.unwrap();
            Ok(info)
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

pub fn inspect_myself() -> Result<ProcessInfo, ()> {
    let mut info = ProcessInfo {
        name: String::new(),
        pid: 0,
        parent: 0,
        state: erhino_shared::proc::ProcessState::Ready,
        permissions: ProcessPermission::Valid.into(),
    };
    let mut name_buffer = [0u8; 256];
    let ret = unsafe { sys_inspect_myself(&mut info, &mut name_buffer) };
    if ret {
        let mut len = 0usize;
        for i in 0..256 {
            if name_buffer[i] == 0 {
                len = i;
                break;
            }
        }
        if len > 0 {
            let name = String::from_utf8(name_buffer[..len].to_vec());
            info.name = name.unwrap();
            Ok(info)
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}
