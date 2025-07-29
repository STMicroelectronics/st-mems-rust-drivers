#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod ucf_entry;
#[cfg(feature = "std")]
pub mod parser;
