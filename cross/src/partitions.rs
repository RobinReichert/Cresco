pub struct PartitionTable {
    pub nvs: core::ops::Range<u32>,
    pub phy_init: core::ops::Range<u32>,
    pub factory: core::ops::Range<u32>,
    pub credentials: core::ops::Range<u32>,
    pub calibration: core::ops::Range<u32>,
}

pub const PARTITIONS: PartitionTable = PartitionTable {
    nvs: 0x9000..0xf000,
    phy_init: 0xf000..0x10000,
    factory: 0x10000..0x208000,
    credentials: 0x208000..0x304000,
    calibration: 0x304000..0x400000,
};
