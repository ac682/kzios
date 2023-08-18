use semver::Version;
use strum::IntoEnumIterator;

use crate::sbi::{
    sbi_get_impl_id, sbi_get_impl_version, sbi_get_mach_arch_id, sbi_get_mach_impl_id,
    sbi_get_mach_vendor_id, sbi_get_spec_version, sbi_probe_extension, SbiExtension,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SbiImplementation {
    BerkeleyBootLoader,
    OpenSBI,
    Xvisor,
    KVM,
    RustSBI,
    Diosix,
    Coffer,
    XenProject,
    PolarFireHartSoftwareServices,
    Unknown(usize),
}

pub struct SbiInfo {
    sepc_ver: Version,
    impl_id: SbiImplementation,
    impl_ver: usize,
    mach_vendor_id: usize,
    mach_arch_id: usize,
    mach_impl_id: usize,
    extensions: u32,
}

impl SbiInfo {
    pub fn new() -> Self {
        let mut extensions = 0u32;
        for extension in SbiExtension::iter() {
            if let Ok(1) = sbi_probe_extension(extension) {
                extensions |= extension.to_index();
            }
        }
        let spec_ver_usize = sbi_get_spec_version().unwrap() as usize;
        let sepc_ver = Version::new(
            ((spec_ver_usize >> 24) & ((1 << 7) - 1)) as u64,
            (spec_ver_usize & ((1 << 24) - 1) )as u64,
            0,
        );
        let impl_id = sbi_get_impl_id().unwrap() as usize;
        let implementation = match impl_id {
            0 => SbiImplementation::BerkeleyBootLoader,
            1 => SbiImplementation::OpenSBI,
            2 => SbiImplementation::Xvisor,
            3 => SbiImplementation::KVM,
            4 => SbiImplementation::RustSBI,
            5 => SbiImplementation::Diosix,
            6 => SbiImplementation::Coffer,
            7 => SbiImplementation::XenProject,
            8 => SbiImplementation::PolarFireHartSoftwareServices,
            _ => SbiImplementation::Unknown(impl_id),
        };
        Self {
            sepc_ver,
            impl_id: implementation,
            impl_ver: sbi_get_impl_version().unwrap() as usize,
            mach_vendor_id: sbi_get_mach_vendor_id().unwrap() as usize,
            mach_arch_id: sbi_get_mach_arch_id().unwrap() as usize,
            mach_impl_id: sbi_get_mach_impl_id().unwrap() as usize,
            extensions,
        }
    }

    pub fn spec_version(&self) -> &Version {
        &self.sepc_ver
    }

    pub fn impl_id(&self) -> SbiImplementation {
        self.impl_id
    }

    pub fn impl_version(&self) -> usize {
        self.impl_ver
    }

    pub fn machine_vendor_id(&self) -> usize {
        self.mach_vendor_id
    }

    pub fn machine_arch_id(&self) -> usize {
        self.mach_arch_id
    }

    pub fn machine_impl_id(&self) -> usize {
        self.mach_impl_id
    }

    pub fn is_extension_supported(&self, eid: SbiExtension) -> bool {
        let id = eid.to_index();
        self.extensions & id == id
    }
}
