//! rusty-leveldb is a reimplementation of LevelDB in pure rust. It depends only on a few crates,
//! and is very close to the original, implementation-wise. The external API is relatively small
//! and should be easy to use.
//!
//! ```
//! use rusty_leveldb::{DB, DBIterator, LdbIterator, Options};
//!
//! let opt = rusty_leveldb::in_memory();
//! let mut db = DB::open("mydatabase", opt).unwrap();
//!
//! db.put(b"Hello", b"World").unwrap();
//! assert_eq!(b"World", db.get(b"Hello").unwrap().as_slice());
//!
//! let mut iter = db.new_iter().unwrap();
//! // Note: For efficiency reasons, it's recommended to use advance() and current() instead of
//! // next() when iterating over many elements.
//! assert_eq!((b"Hello".to_vec(), b"World".to_vec()), iter.next().unwrap());
//!
//! db.delete(b"Hello").unwrap();
//! db.flush().unwrap();
//! ```
//!
#![cfg_attr(feature = "mesalock_sgx", no_std)]
#![allow(dead_code)]
#![allow(clippy::all)]

#[cfg(feature = "mesalock_sgx")]
#[macro_use]
extern crate sgx_tstd as std;

#[macro_use]
extern crate sgx_tunittest;

extern crate protected_fs;
extern crate sgx_libc as libc;
extern crate sgx_trts;
extern crate sgx_types;

extern crate crc;
extern crate integer_encoding;
extern crate rand;
extern crate snap;



pub mod block;
pub mod block_builder;
pub mod blockhandle;
pub mod cache;
pub mod cmp;
pub mod disk_env;
mod env;
mod env_common;
mod error;
pub mod filter;
pub mod filter_block;
#[macro_use]
mod infolog;
pub mod key_types;
pub mod log;
pub mod mem_env;
pub mod memtable;
pub mod merging_iter;
mod options;
pub mod skipmap;
pub mod snapshot;
pub mod table_block;
pub mod table_builder;
pub mod table_cache;
pub mod table_reader;
pub mod test_util;
pub mod types;
pub mod version;
pub mod version_edit;
pub mod version_set;
pub mod write_batch;

pub mod db_impl;
pub mod db_iter;

pub use crate::cmp::{Cmp, DefaultCmp};
pub use crate::db_iter::DBIterator;
pub use crate::env::Env;
pub use crate::error::{Result, Status, StatusCode};
pub use crate::filter::{BloomPolicy, FilterPolicy};
pub use crate::mem_env::MemEnv;
pub use crate::options::{in_memory, CompressionType, Options};
pub use crate::skipmap::SkipMap;
pub use crate::types::LdbIterator;
pub use crate::write_batch::WriteBatch;
pub use db_impl::DB;
pub use disk_env::PosixDiskEnv;
