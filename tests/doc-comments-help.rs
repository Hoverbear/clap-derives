// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>,
// Kevin Knapp (@kbknapp) <kbknapp@gmail.com>, and
// Andrew Hobden (@hoverbear) <andrew@hoverbear.org>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// This work was derived from Structopt (https://github.com/TeXitoi/structopt)
// commit#ea76fa1b1b273e65e3b0b1046643715b49bec51f which is licensed under the
// MIT/Apache 2.0 license.

#[macro_use]
extern crate clap;

mod utils;

use clap::Clap;
use utils::*;

#[test]
fn doc_comments() {
    /// Lorem ipsum
    #[derive(Clap, PartialEq, Debug)]
    struct LoremIpsum {
        /// Fooify a bar
        /// and a baz
        #[clap(short, long)]
        foo: bool,
    }

    let help = get_long_help::<LoremIpsum>();
    assert!(help.contains("Lorem ipsum"));
    assert!(help.contains("Fooify a bar and a baz"));
}

#[test]
fn help_is_better_than_comments() {
    /// Lorem ipsum
    #[derive(Clap, PartialEq, Debug)]
    #[clap(name = "lorem-ipsum", about = "Dolor sit amet")]
    struct LoremIpsum {
        /// Fooify a bar
        #[clap(short, long, help = "DO NOT PASS A BAR UNDER ANY CIRCUMSTANCES")]
        foo: bool,
    }

    let help = get_long_help::<LoremIpsum>();
    assert!(help.contains("Dolor sit amet"));
    assert!(!help.contains("Lorem ipsum"));
    assert!(help.contains("DO NOT PASS A BAR"));
}

#[test]
fn empty_line_in_doc_comment_is_double_linefeed() {
    /// Foo.
    ///
    /// Bar
    #[derive(Clap, PartialEq, Debug)]
    #[clap(name = "lorem-ipsum", no_version)]
    struct LoremIpsum {}

    let help = get_long_help::<LoremIpsum>();
    assert!(help.starts_with("lorem-ipsum \nFoo.\n\nBar\n\nUSAGE:"));
}

#[test]
fn field_long_doc_comment_both_help_long_help() {
    /// Lorem ipsumclap
    #[derive(Clap, PartialEq, Debug)]
    #[clap(name = "lorem-ipsum", about = "Dolor sit amet")]
    struct LoremIpsum {
        /// DO NOT PASS A BAR UNDER ANY CIRCUMSTANCES.
        ///
        /// Or something else
        #[clap(long)]
        foo: bool,
    }

    let short_help = get_help::<LoremIpsum>();
    let long_help = get_long_help::<LoremIpsum>();

    assert!(short_help.contains("CIRCUMSTANCES"));
    assert!(!short_help.contains("CIRCUMSTANCES."));
    assert!(!short_help.contains("Or something else"));
    assert!(long_help.contains("DO NOT PASS A BAR UNDER ANY CIRCUMSTANCES"));
    assert!(long_help.contains("Or something else"));
}

#[test]
fn top_long_doc_comment_both_help_long_help() {
    /// Lorem ipsumclap
    #[derive(Clap, Debug)]
    #[clap(name = "lorem-ipsum", about = "Dolor sit amet")]
    struct LoremIpsum {
        #[clap(subcommand)]
        foo: SubCommand,
    }

    #[derive(Clap, Debug)]
    pub enum SubCommand {
        /// DO NOT PASS A BAR UNDER ANY CIRCUMSTANCES
        ///
        /// Or something else
        Foo {
            #[clap(help = "foo")]
            bars: Vec<String>,
        },
    }

    let short_help = get_help::<LoremIpsum>();
    let long_help = get_subcommand_long_help::<LoremIpsum>("foo");

    assert!(!short_help.contains("Or something else"));
    assert!(long_help.contains("DO NOT PASS A BAR UNDER ANY CIRCUMSTANCES"));
    assert!(long_help.contains("Or something else"));
}
