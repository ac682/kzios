use crate::{board::BoardInfo, peripheral::aclint::Aclint};

pub mod aclint;
pub mod plic;

static mut ACLINT: Option<Aclint> = None;

pub fn init(info: &BoardInfo) {
    unsafe {
        ACLINT = Some(Aclint::new(
            info.mswi_address,
            info.mtimer_address,
            info.mtime_address,
        ))
    }
}

pub fn aclint() -> &'static mut Aclint {
    unsafe{
        if let Some(aclint) = &mut ACLINT{
            aclint
        }else{
            panic!("unavailable");
        }
    }
}
