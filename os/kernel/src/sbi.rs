use core::arch::asm;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;

#[repr(isize)]
#[derive(Debug, FromPrimitive, ToPrimitive)]
pub enum SbiError {
    Success = 0,
    Failed = -1,
    NotSupported = -2,
    InvalidParameter = -3,
    Denied = -4,
    InvalidAddress = -5,
    AlreadyAvailable = -6,
    AlreadyStarted = -7,
    AlreadyStopped = -8,
    NoSharedMemory = -9,
}

pub type SbiResult = Result<isize, SbiError>;

#[repr(usize)]
#[allow(unused)]
pub enum SbiExtension {
    LegacySetTimer = 0x0,
    LegacyConsolePutchar = 0x01,
    LegacyConsoleGetchar = 0x02,
    LegacyClearIPI = 0x03,
    LegacySendIPI = 0x04,
    LegacyRemoteFenceI = 0x05,
    LegacyRemoteSFenceVma = 0x06,
    LegacyRemoteSFenceVmaAsid = 0x07,
    LegacySystemShutdown = 0x08,
    Time = 0x54494D45,
    InterProcessInterrupt = 0x735049,
    RemoteFence = 0x52464E43,
    HartStateManagement = 0x48534D,
    SystemReset = 0x53525354,
    PerformanceMonitorUnit = 0x504D55,
    DebugConsole = 0x4442434E,
    SystemSuspend = 0x53555350,
    CollaborativeProcessorPerformanceControl = 0x43505043,
    NestedAcceleration = 0x4E41434C,
    StealTimeAccounting = 0x535441,
    Base = 0x10,
}

static mut TIME_SUPPORTED: bool = false;
static mut DEBUG_CONSOLE_SUPPORTED: bool = false;

#[inline]
fn raw_call(
    eid: SbiExtension,
    fid: usize,
    arg0: usize,
    arg1: usize,
    arg2: usize,
) -> (isize, isize) {
    let mut error: isize;
    let mut value: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") arg0 => error,
            inlateout("a1") arg1 => value,
            in("a2") arg2,
            in("a6") fid,
            in("a7") eid as usize
        );
    }
    (error, value)
}

fn legacy_call(eid: SbiExtension, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret: usize;
    unsafe {
        asm!("ecall",
            inlateout("a0") arg0 => ret,
            in("a1") arg1,
            in("a2") arg2,
            in("a7") eid as usize);
        ret
    }
}

fn sbi_call(eid: SbiExtension, fid: usize, arg0: usize, arg1: usize, arg2: usize) -> SbiResult {
    let (error, value) = raw_call(eid, fid, arg0, arg1, arg2);
    if error == 0 {
        Ok(value)
    } else {
        if let Some(res) = SbiError::from_isize(error) {
            Err(res)
        } else {
            Err(SbiError::NotSupported)
        }
    }
}

pub fn legacy_set_timer(time: usize) {
    legacy_call(SbiExtension::LegacySetTimer, time, 0, 0);
}

pub fn legacy_console_putchar(char: u8) {
    legacy_call(SbiExtension::LegacyConsolePutchar, char as usize, 0, 0);
}

pub fn debug_console_write(text: &str) -> SbiResult {
    let ptr = text.as_ptr();
    let count = text.len();
    sbi_call(SbiExtension::DebugConsole, 0, count, ptr as usize, 0)
}

pub fn debug_console_write_byte(byte: u8) -> SbiResult {
    sbi_call(SbiExtension::DebugConsole, 2, byte as usize, 0 as usize, 0)
}

pub fn hart_start(hartid: usize, start_addr: usize, opaque: usize) -> SbiResult {
    sbi_call(
        SbiExtension::HartStateManagement,
        0,
        hartid,
        start_addr,
        opaque,
    )
}

pub fn hart_stop(hartid: usize) -> SbiResult {
    sbi_call(SbiExtension::HartStateManagement, 1, hartid, 0, 0)
}

pub fn hart_get_status(hartid: usize) -> SbiResult {
    sbi_call(SbiExtension::HartStateManagement, 2, hartid, 0, 0)
}

pub fn hart_suspend(hartid: usize, resume_addr: usize, opaque: usize) -> SbiResult {
    sbi_call(
        SbiExtension::HartStateManagement,
        3,
        hartid,
        resume_addr,
        opaque,
    )
}

pub fn set_timer(time: usize) -> SbiResult {
    sbi_call(SbiExtension::Time, 0, time, 0, 0)
}

/// Value: Returns 0 if the given SBI extension ID (EID) is not available, or 1 if it is available unless defined as
/// any other non-zero value by the implementation.
pub fn probe_extension(eid: SbiExtension) -> SbiResult {
    sbi_call(SbiExtension::Base, 3, eid as usize, 0, 0)
}

pub fn send_ipi(hart_mask: usize, hart_mask_base: isize) -> SbiResult {
    sbi_call(
        SbiExtension::InterProcessInterrupt,
        0x0,
        hart_mask,
        hart_mask_base as usize,
        0,
    )
}

pub fn init() {
    if let Ok(res) = probe_extension(SbiExtension::DebugConsole) {
        unsafe {
            DEBUG_CONSOLE_SUPPORTED = res != 0;
        }
    }
    if let Ok(res) = probe_extension(SbiExtension::Time) {
        unsafe {
            TIME_SUPPORTED = res != 0;
        }
    }
}

pub fn is_debug_console_supported() -> bool {
    unsafe { DEBUG_CONSOLE_SUPPORTED }
}

pub fn is_time_supported() -> bool {
    unsafe { TIME_SUPPORTED }
}
