use super::InteriorLock;

pub struct OptimisticLock {}

impl InteriorLock for OptimisticLock {
    fn is_locked(&self) -> bool {
        false
    }

    fn lock(&mut self) {
        ()
    }

    fn try_lock(&mut self) -> bool {
        true
    }

    fn unlock(&mut self) {
        ()
    }
}

impl OptimisticLock {
    pub const fn new() -> Self {
        Self {}
    }
}
