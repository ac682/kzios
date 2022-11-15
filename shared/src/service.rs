use num_derive::{FromPrimitive, ToPrimitive};



#[repr(usize)]
#[derive(FromPrimitive, ToPrimitive)]
/// Predefined service id
pub enum ServiceId {
    /// FS
    FileSystem = 1,
    /// TS
    TimerService = 2,
    /// Id must be less than this or defined as user registered service
    Reserved = 3,
}
