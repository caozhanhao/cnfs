//! CNFS Virtual file system abstraction
//!
//! The virtual file system abstraction is written for CNSS Recruit 2024
//!
#![no_std]
#![deny(missing_docs)]
#![deny(warnings)]
extern crate alloc;
mod error;
mod usrlyr;
mod vfs;
mod sync;
mod config;

pub use error::*;
pub use usrlyr::*;
pub use vfs::*;
