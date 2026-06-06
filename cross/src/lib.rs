#![no_std]
#![feature(impl_trait_in_assoc_type)]

pub mod dhcp;
pub mod dns;
pub mod partitions;
pub mod shared_flash;
pub mod storage;
pub mod web;
pub mod wifi;

#[macro_export]
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}
