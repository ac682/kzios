use super::InteriorLock;

pub struct OptimisticLock {}

impl InteriorLock for OptimisticLock {
    fn is_locked(&self) -> bool {
        false
    }

    fn lock(&self) {
        ()
    }

    fn try_lock(&self) -> bool {
        true
    }

    fn unlock(&self) {
        ()
    }
}

impl OptimisticLock {
    pub const fn new() -> Self {
        Self {}
    }
}
