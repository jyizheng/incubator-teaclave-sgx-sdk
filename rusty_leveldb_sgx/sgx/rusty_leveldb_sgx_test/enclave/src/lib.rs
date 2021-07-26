// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#![crate_name = "helloworldsampleenclave"]
#![crate_type = "staticlib"]
#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_tunittest;

use sgx_tunittest::*;
use sgx_types::*;
use std::io::{self, ErrorKind};
use std::slice;
use std::string::String;

use std::time::*;
use std::untrusted::time::InstantEx;

extern crate rand;
extern crate rusty_leveldb;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::iter;

use rusty_leveldb::CompressionType;
use rusty_leveldb::Options;
use rusty_leveldb::DB;
use rusty_leveldb::types::current_key_val;
use rusty_leveldb::LdbIterator;

use std::boxed::Box;
use std::error::Error;
use std::untrusted::fs;

const KEY_LEN: usize = 64;
const VAL_LEN: usize = 300;

fn gen_string(len: usize) -> String {
    let mut rng = rand::thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(len)
        .collect()
}

fn int_to_string(i: usize) -> String {
    format!("{:064}", i)
}

#[warn(dead_code)]
fn fill_db(db: &mut DB, entries: usize) -> Result<(), Box<dyn Error>> {
    for i in 0..entries {
        let (k, v) = (gen_string(KEY_LEN), gen_string(VAL_LEN));
        db.put(k.as_bytes(), v.as_bytes())?;
        if i % 1000 == 0 {
            db.flush()?;

            let v2 = db
                .get(k.as_bytes())
                .ok_or_else(|| Box::new(io::Error::new(ErrorKind::NotFound, "Key not found")))?;
            assert_eq!(&v.as_bytes()[..], &v2[..]);

            db.delete(k.as_bytes())?;
            assert_eq!(true, db.get(k.as_bytes()).is_none());
        }

        if i % 100 == 0 {
            db.flush()?;
        }
    }
    Ok(())
}

fn bench_write(num_mb: usize, is_seq: bool) -> Result<(), Box<dyn Error>> {
    let key = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x0f, 0x0e, 0x0d, 0x0c, 0x0b, 0x0a, 0x09,
        0x08,
    ];
    let mut opt = Options::new_disk_db_with(key);
    opt.compression_type = CompressionType::CompressionSnappy;

    let a = Instant::now();

    let mut db = DB::open("/tmp/leveldb_testdb", opt)?;
    let entries = 2748 * num_mb;

    for i in 0..entries {
        let (k, v) = if is_seq {
            (gen_string(KEY_LEN), gen_string(VAL_LEN))
        } else {
            (int_to_string(i), gen_string(VAL_LEN))
        };
        db.put(k.as_bytes(), v.as_bytes())?;
        if i % 100 == 0 || i == entries - 1 {
            db.flush()?;
        }
    }
    drop(db);

    let b = Instant::now();
    let dur = b.duration_since(a);
    println!("dur={:?}", dur);
    fs::remove_dir_all("/tmp/leveldb_testdb").expect("Cannot remove directory");
    Ok(())
}

fn bench_rand_read_or_delete(
    num_mb: usize, 
    is_del: bool
) -> Result<(), Box<dyn Error>> {
    let key = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x0f, 0x0e, 0x0d, 0x0c, 0x0b, 0x0a, 0x09,
        0x08,
    ];
    let mut opt = Options::new_disk_db_with(key);
    opt.compression_type = CompressionType::CompressionSnappy;

    let mut db = DB::open("/tmp/leveldb_testdb", opt)?;
    let entries = 2748 * num_mb;

    for i in 0..entries {
        let (k, v) = (int_to_string(i), gen_string(VAL_LEN));
        db.put(k.as_bytes(), v.as_bytes())?;
        if i % 100 == 0 || i == entries - 1 {
            db.flush()?;
        }
    }

    let a = Instant::now();
    let run = 10000;
    for i in 0..run {
        let mut rng = rand::thread_rng();
        let x: usize = rng.gen::<usize>() % entries;
        let key = int_to_string(x);
        if is_del {
            db.delete(key.as_bytes())?;
            if i % 100 == 0 || i == run-1 {
                db.flush()?;
            }
        } else {
            let _val = db
                .get(key.as_bytes())
                .ok_or_else(|| Box::new(io::Error::new(ErrorKind::NotFound, "Key not found")))?;
        }
    }
    let b = Instant::now();
    let dur = b.duration_since(a);
    println!("dur={:?}", dur.as_millis());

    drop(db);
    fs::remove_dir_all("/tmp/leveldb_testdb").expect("Cannot remove directory");
    Ok(())
}

fn bench_scan(
    num_mb: usize, 
) -> Result<(), Box<dyn Error>> {
    let key = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x0f, 0x0e, 0x0d, 0x0c, 0x0b, 0x0a, 0x09,
        0x08,
    ];
    let mut opt = Options::new_disk_db_with(key);
    opt.compression_type = CompressionType::CompressionSnappy;

    let mut db = DB::open("/tmp/leveldb_testdb", opt)?;
    let entries = 2748 * num_mb;

    for i in 0..entries {
        let (k, v) = (int_to_string(i), gen_string(VAL_LEN));
        db.put(k.as_bytes(), v.as_bytes())?;
        if i % 100 == 0 || i == entries - 1 {
            db.flush()?;
        }
    }

    let a = Instant::now();
    let mut iter = db.new_iter().unwrap();

    let run = 500;
    let scan_length = 1000;
    for _i in 0..run {
        let mut rng = rand::thread_rng();
        let x: usize = rng.gen::<usize>() % (entries-scan_length);
        let key = int_to_string(x);
        iter.seek(key.as_bytes());
        for _j in 0..scan_length {
            assert!(iter.advance());
            let (_k, _v) = current_key_val(&iter).unwrap();
        }
    }
    let b = Instant::now();
    let dur = b.duration_since(a);
    println!("dur={:?}", dur.as_millis());

    drop(db);
    fs::remove_dir_all("/tmp/leveldb_testdb").expect("Cannot remove directory");
    Ok(())
}


#[no_mangle]
pub extern "C" fn bench_kv(
    data_size: *const u8,
    len1: usize,
    bench_name: *const u8,
    len2: usize,
) -> sgx_status_t {
    // get data size
    let str_slice = unsafe { slice::from_raw_parts(data_size, len1) };
    let new_string = String::from_utf8(str_slice.to_vec()).unwrap();
    let size = new_string.parse::<usize>().unwrap();
    // get bench name
    let str_slice = unsafe { slice::from_raw_parts(bench_name, len2) };
    let name = String::from_utf8(str_slice.to_vec()).unwrap();

    // print the input argument
    println!("size={}MB, name={}", size, name);
    let res = match &name[..] {
        "rand_write" => bench_write(size, false),
        "seq_write" => bench_write(size, true),
        "rand_read" => bench_rand_read_or_delete(size, false),
        "rand_delete" => bench_rand_read_or_delete(size, true),
        "scan" => bench_scan(size),
        _ => Ok(()),
    };

    match res {
        Err(e) => {
            println!("Error:{:?}", e);
            sgx_status_t::SGX_ERROR_UNEXPECTED
        }
        Ok(_) => sgx_status_t::SGX_SUCCESS,
    }
}

/*
fn unit_test() {
    rsgx_unit_tests!(
        rusty_leveldb::block::tests::run_tests,
        rusty_leveldb::block_builder::tests::run_tests,
        rusty_leveldb::blockhandle::tests::run_tests,
        rusty_leveldb::cache::tests::run_tests,
        rusty_leveldb::cmp::tests::run_tests,
        rusty_leveldb::db_impl::tests::run_tests,
        rusty_leveldb::db_iter::tests::run_tests,
        rusty_leveldb::disk_env::tests::run_tests,
        rusty_leveldb::filter::tests::run_tests,
        rusty_leveldb::filter_block::tests::run_tests,
        rusty_leveldb::key_types::tests::run_tests,
        rusty_leveldb::log::tests::run_tests,
        rusty_leveldb::mem_env::tests::run_tests,
        rusty_leveldb::memtable::tests::run_tests,
        rusty_leveldb::merging_iter::tests::run_tests,
        rusty_leveldb::skipmap::tests::run_tests,
        rusty_leveldb::snapshot::tests::run_tests,
        rusty_leveldb::table_builder::tests::run_tests,
        rusty_leveldb::table_cache::tests::run_tests,
        rusty_leveldb::test_util::tests::run_tests,
        rusty_leveldb::table_reader::tests::run_tests,
        rusty_leveldb::types::tests::run_tests,
        rusty_leveldb::version::tests::run_tests,
        rusty_leveldb::version_edit::tests::run_tests,
        rusty_leveldb::version_set::tests::run_tests,
        rusty_leveldb::write_batch::tests::run_tests,
    );
}
*/
