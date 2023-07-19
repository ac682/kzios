// Sv39 only

use erhino_shared::mem::PageNumber;
use flagset::{flags, FlagSet};

pub enum PageTableError {
    EntryNotFound,
    EntryDefinitionConflicts,
    EntryUndefined,
    EntryNotLeaf,
    EntryNotBranch,
    WrongLevel,
    PhysicalPageNumberUnaligned,
}

flags! {
    pub enum PageTableEntryFlag: u64{
        Valid = 0b1,
        Readable = 0b10,
        Writeable = 0b100,
        Executable = 0b1000,
        User = 0b1_0000,
        Global = 0b10_0000,
        Accessed = 0b100_0000,
        Dirty = 0b1000_0000,

        UserReadWrite = (PageTableEntryFlag::User | PageTableEntryFlag::Readable | PageTableEntryFlag::Writeable | PageTableEntryFlag::Valid).bits(),
    }
}

