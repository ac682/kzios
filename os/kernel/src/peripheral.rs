use crate::{
    board::BoardInfo,
    peripheral::aclint::Aclint,
    sync::{
        optimistic::{OptimisticLock, OptimisticLockGuard},
        Lock,
    },
};

pub mod aclint;
pub mod plic;

static mut ACLINT: OptimisticLock<Aclint> = OptimisticLock::empty();

pub fn init(info: &BoardInfo) {
    unsafe { ACLINT.put(Aclint::new(info.mswi_address, info.mtimer_address)) }
}

pub fn aclint() -> OptimisticLockGuard<Aclint> {
    unsafe { ACLINT.lock() }
}
