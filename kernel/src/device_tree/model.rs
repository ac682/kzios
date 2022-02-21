use core::{
    fmt::{Debug},
    mem::{size_of, size_of_val},
    slice::from_raw_parts,
};

const MAGIC: u32 = 0xd00dfeed;

// #[repr(C)]
// #[derive(Debug)]
// pub struct DtbHeader {
//     pub magic: u32,
//     pub total_size: u32,
//     pub off_dt_struct: u32,
//     pub off_dt_strings: u32,
//     pub off_mem_rsvmap: u32,
//     pub version: u32,
//     pub last_comp_version: u32,
//     pub boot_cpuid_phys: u32,
//     pub size_dt_strings: u32,
//     pub size_dt_struct: u32,
// }

#[repr(C)]
#[derive(Debug)]
pub struct DtbHeader {
    pub magic: BigU32,
    pub total_size: BigU32,
    pub off_dt_struct: BigU32,
    pub off_dt_strings: BigU32,
    pub off_mem_rsvmap: BigU32,
    pub version: BigU32,
    pub last_comp_version: BigU32,
    pub boot_cpuid_phys: BigU32,
    pub size_dt_strings: BigU32,
    pub size_dt_struct: BigU32,
}

#[repr(C)]
pub struct BigU32(u8, u8, u8, u8);

impl Debug for BigU32 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let num: u32 = self.into();
        f.write_fmt(format_args!("BigU32(0x{:x})", num))
    }
}

impl From<&BigU32> for u32 {
    fn from(data: &BigU32) -> u32 {
        u32::from_be_bytes([data.0, data.1, data.2, data.3])
    }
}

pub struct DtbNodeHeader {
    tag: u32,
    name_ptr: usize,
}

enum DtbNodeHeaderTag {
    BeginNode = 0x1,
    EndNode = 0x2,
    Property = 0x3,
    Nop = 0x4,
    End = 0x9,
}

pub struct DtbProperty {
    tag: u32,
    length: u32,
    off_name: u32,
    data_ptr: usize,
}

pub struct DeviceTree {
    pub buffer: &'static [u8],
    pub header: &'static DtbHeader,
    pub fill_area: &'static [u8],
    dt_struct: &'static [u8],
    dt_strings: &'static [u8],
}

impl DeviceTree {
    pub fn new(addr: usize) -> Self {
        let magic = unsafe { u32::from_be(*(addr as *const u32)) };
        if magic != MAGIC {
            panic!("Magic number not match({}): {}", MAGIC, magic);
        }
        let size = unsafe { u32::from_be(*((addr + size_of::<u32>()) as *const u32)) };

        let header = unsafe { &*(addr as *const DtbHeader) };
        println!("{:?}", header);
        let header_size: usize = size_of_val(&header);
        DeviceTree {
            buffer: unsafe { from_raw_parts(addr as *const u8, u32::from(&header.total_size) as usize) },
            fill_area: unsafe {
                from_raw_parts(
                    (addr + header_size) as *const u8,
                    u32::from(&header.off_dt_struct) as usize - header_size,
                )
            },
            dt_struct: unsafe {
                from_raw_parts(
                    (addr + u32::from(&header.off_dt_struct) as usize) as *const u8,
                    u32::from(&header.size_dt_struct) as usize,
                )
            },
            dt_strings: unsafe {
                from_raw_parts(
                    (addr + u32::from(&header.off_dt_strings) as usize) as *const u8,
                    u32::from(&header.size_dt_strings) as usize,
                )
            },
            header: header,
        }
    }
}
