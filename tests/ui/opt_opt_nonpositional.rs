// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate clap;

use clap::Clap;

#[derive(Clap, Debug)]
#[clap(name = "basic")]
struct Opt {
    n: Option<Option<u32>>,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
