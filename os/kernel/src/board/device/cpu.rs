use dtb_parser::{node::DeviceTreeNode, prop::PropertyValue, traits::FindPropertyValue};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MmuType {
    Bare,
    Sv32,
    Sv39,
    Sv48,
    Sv57,
}

pub struct Isa {
    pub len: usize,
    pub i: bool,
    pub m: bool,
    pub a: bool,
    pub f: bool,
    pub d: bool,
    pub c: bool,
}

impl Isa {
    pub fn from_str(s: &str) -> Option<Self> {
        let isa = if let Some(i) = s.find('_') {
            &s[..i]
        } else {
            s
        };
        let len = if isa.starts_with("rv64") {
            64
        } else if isa.starts_with("rv32") {
            32
        } else {
            return None;
        };
        let i = isa.contains('i');
        let m = isa.contains('m');
        let a = isa.contains('a');
        let f = isa.contains('f');
        let d = isa.contains('d');
        let c = isa.contains('c');
        Some(Self {
            len,
            i,
            m,
            a,
            f,
            d,
            c,
        })
    }
}

pub struct Cpu {
    hartid: usize,
    freq: usize,
    mmu: MmuType,
    isa: Isa,
}

impl Cpu {
    pub fn from_device_node(
        node: &DeviceTreeNode,
        timebase_frequency: &Option<usize>,
    ) -> Option<Self> {
        if let Some(PropertyValue::String(device_type)) = node.value("device_type") {
            if *device_type == "cpu" {
                let id = if let Some(PropertyValue::Address(i, _)) = node.value("reg") {
                    *i
                } else {
                    return None;
                };
                let mmu = if let Some(PropertyValue::String(m)) = node.value("mmu-type") {
                    match (*m).as_str() {
                        "riscv,sv32" => MmuType::Sv32,
                        "riscv,sv39" => MmuType::Sv39,
                        "riscv,sv48" => MmuType::Sv48,
                        "riscv,sv57" => MmuType::Sv57,
                        _ => MmuType::Bare,
                    }
                } else {
                    MmuType::Bare
                };
                let isa = if let Some(PropertyValue::String(isa_str)) = node.value("riscv,isa") {
                    if let Some(i) = Isa::from_str((*isa_str).as_str()) {
                        i
                    } else {
                        return None;
                    }
                } else {
                    return None;
                };
                let freq = if let Some(PropertyValue::Integer(f)) = node.value("clock-frequency") {
                    *f as usize
                } else {
                    if let Some(t) = timebase_frequency {
                        *t
                    } else {
                        return None;
                    }
                };
                Some(Self {
                    hartid: id as usize,
                    freq,
                    isa,
                    mmu,
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn id(&self) -> usize {
        self.hartid
    }

    pub fn freq(&self) -> usize {
        self.freq
    }

    pub fn mmu(&self) -> MmuType {
        self.mmu
    }

    pub fn isa(&self) -> &Isa {
        &self.isa
    }
}
