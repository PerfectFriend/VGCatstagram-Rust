//! # VanGoghCat P2P
//!
//! CatP2P daemon, contract negotiations, Simple Messaging Protocol (SMP), and XFTP swarms.

pub mod contract;
pub mod smp;
pub mod xftp;
pub mod daemon;
pub mod nostr;

pub use contract::*;
pub use smp::*;
pub use xftp::*;
pub use daemon::*;
pub use nostr::*;
