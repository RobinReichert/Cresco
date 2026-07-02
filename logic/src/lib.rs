#![cfg_attr(not(test), no_std)]

pub mod blackboard;
pub mod calibration;
pub mod config;
pub mod dhcp;
pub mod dns;
pub mod pid;
pub mod wifi;

pub type Float = f64;
