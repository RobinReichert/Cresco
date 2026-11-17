#![cfg_attr(not(test), no_std)]

pub mod blackboard;
pub mod config;
pub mod dhcp;
pub mod dns;
pub mod equations;
pub mod measurement;
pub mod pid;
pub mod wifi;

pub type Float = f64;
