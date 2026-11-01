//! # VanGoghCat P2P
//!
//! CatP2P daemon, contract negotiations, Simple Messaging Protocol (SMP), and XFTP swarms.

pub mod contract;
pub mod daemon;
pub mod discovery;
pub mod fec;
pub mod nostr;
pub mod smp;
pub mod xftp;

pub use contract::*;
pub use daemon::*;
pub use discovery::*;
pub use fec::*;
pub use nostr::*;
pub use smp::*;
pub use xftp::*;
