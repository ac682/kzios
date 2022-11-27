use erhino_shared::service::{ServiceId, Sid};

use crate::call::sys_service_register;

/// Only system service can call this
pub fn register(service: ServiceId) -> Result<Sid, ()> {
    // 返回值应该指出是权限不足还是其他原因，但是目前只有权限不足一个原因
    let sid = service as Sid;
    let success = unsafe { sys_service_register(sid) };
    if success {
        Ok(sid)
    } else {
        Err(())
    }
}
