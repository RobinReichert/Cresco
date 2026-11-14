pub struct PartitionTable {
    pub nvs: core::ops::Range<u32>,
    pub phy_init: core::ops::Range<u32>,
    pub factory: core::ops::Range<u32>,
    pub storage: core::ops::Range<u32>,
}

pub const PARTITIONS: PartitionTable = PartitionTable {
    nvs: 0x9000..0xf000,
    phy_init: 0xf000..0x10000,
    factory: 0x10000..0x208000,
    storage: 0x208000..0x400000,
};
