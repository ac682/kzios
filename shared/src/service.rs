use num_derive::{FromPrimitive, ToPrimitive};

/// Service id, treated as special process id
pub type Sid = usize;

#[repr(usize)]
#[derive(FromPrimitive, ToPrimitive)]
/// Predefined service id
pub enum ServiceId {
    /// FS
    FileSystem = 1,
    /// TS
    TimerService = 2,
    /// Id must be less than this or defined as user registered service
    /// 1-128 is reserved for system services
    Reserved = 128,
}
