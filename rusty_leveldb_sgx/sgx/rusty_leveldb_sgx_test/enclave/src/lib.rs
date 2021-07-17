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

use sgx_types::*;
use std::string::String;
use std::vec::Vec;
use std::io::{self, Write, ErrorKind};
use std::slice;
use sgx_tunittest::*;

extern crate rand;
extern crate rusty_leveldb;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::iter;

use rusty_leveldb::CompressionType;
use rusty_leveldb::Options;
use rusty_leveldb::DB;

use std::untrusted::fs;
use std::error::Error;
use std::boxed::Box;

const KEY_LEN: usize = 16;
const VAL_LEN: usize = 48;

fn gen_string(len: usize) -> String {
    let mut rng = rand::thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(len)
        .collect()
}

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

fn bench() {
    let key = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x0f, 0x0e, 0x0d, 0x0c, 0x0b, 0x0a, 0x09,
        0x08,
    ];
    let mut opt = Options::new_disk_db_with(key);
    opt.compression_type = CompressionType::CompressionSnappy;

    println!("{}", "before db open");
    let mut db = DB::open("/tmp/leveldb_testdb", opt).unwrap();

    println!("{}", "db open is done");
    fill_db(&mut db, 100).unwrap();

    drop(db);

    println!("{}", "before db remove");
    fs::remove_dir_all("/tmp/leveldb_testdb").expect("Cannot remove directory");
    println!("{}", "db remove is done");
}

#[no_mangle]
pub extern "C" fn say_something(some_string: *const u8, some_len: usize) -> sgx_status_t {

    let str_slice = unsafe { slice::from_raw_parts(some_string, some_len) };
    let _ = io::stdout().write(str_slice);

    // A sample &'static string
    let rust_raw_string = "This is a in-Enclave ";
    // An array
    let word:[u8;4] = [82, 117, 115, 116];
    // An vector
    let word_vec:Vec<u8> = vec![32, 115, 116, 114, 105, 110, 103, 33];

    // Construct a string from &'static string
    let mut hello_string = String::from(rust_raw_string);

    // Iterate on word array
    for c in word.iter() {
        hello_string.push(*c as char);
    }

    // Rust style convertion
    hello_string += String::from_utf8(word_vec).expect("Invalid UTF-8")
                                               .as_str();

    // Ocall to normal world for output
    println!("{}", &hello_string);

    bench();

    sgx_status_t::SGX_SUCCESS
}

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
