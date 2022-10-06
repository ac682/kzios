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

static ACLINT: OptimisticLock<Aclint> = OptimisticLock::empty();

pub fn init(info: BoardInfo) {
    ACLINT.put(Aclint::new(info.mswi_address, info.mtimer_address));
}

pub fn aclint() -> OptimisticLockGuard<Aclint> {
    ACLINT.lock()
}
