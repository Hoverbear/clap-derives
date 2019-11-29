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

use clap::Clap;

#[test]
fn commets_intead_of_actual_help() {
    use clap::IntoApp;

    /// Lorem ipsum
    #[derive(Clap, PartialEq, Debug)]
    struct LoremIpsum {
        /// Fooify a bar
        /// and a baz
        #[clap(short, long)]
        foo: bool,
    }

    let mut output = Vec::new();
    LoremIpsum::into_app().write_long_help(&mut output).unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains("Lorem ipsum"));
    assert!(output.contains("Fooify a bar and a baz"));
}

#[test]
fn help_is_better_than_comments() {
    use clap::IntoApp;
    /// Lorem ipsum
    #[derive(Clap, PartialEq, Debug)]
    #[clap(name = "lorem-ipsum", about = "Dolor sit amet")]
    struct LoremIpsum {
        /// Fooify a bar
        #[clap(short, long, help = "DO NOT PASS A BAR UNDER ANY CIRCUMSTANCES")]
        foo: bool,
    }

    let mut output = Vec::new();
    LoremIpsum::into_app().write_long_help(&mut output).unwrap();
    let output = String::from_utf8(output).unwrap();

    for line in output.split("\n") {
        println!("{:#?}", line);
    }

    assert!(output.contains("Dolor sit amet"));
    assert!(!output.contains("Lorem ipsum"));
    assert!(output.contains("DO NOT PASS A BAR"));
}

#[test]
fn empty_line_in_doc_comment_is_double_linefeed() {
    use clap::IntoApp;
    /// Foo.
    ///
    /// Bar
    #[derive(Clap, PartialEq, Debug)]
    #[clap(name = "lorem-ipsum", no_version)]
    struct LoremIpsum {}

    let mut output = Vec::new();
    LoremIpsum::into_app().write_long_help(&mut output).unwrap();
    let output = String::from_utf8(output).unwrap();

    println!("{}", output);
    assert!(output.starts_with("lorem-ipsum \nFoo.\n\nBar\n\nUSAGE:"));
}

#[test]
fn splits_flag_doc_comment_between_short_and_long() {
    use clap::IntoApp;
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

    let mut app = LoremIpsum::into_app();

    let short_help = {
        let mut buffer = Vec::new();
        app.write_help(&mut buffer).ok();

        String::from_utf8(buffer).unwrap()
    };

    let long_help = {
        let mut buffer = Vec::new();
        app.write_long_help(&mut buffer).ok();

        String::from_utf8(buffer).unwrap()
    };

    assert!(short_help.contains("CIRCUMSTANCES"));
    assert!(!short_help.contains("CIRCUMSTANCES."));
    assert!(!short_help.contains("Or something else"));
    assert!(long_help.contains("DO NOT PASS A BAR UNDER ANY CIRCUMSTANCES"));
    assert!(long_help.contains("Or something else"));
}

#[test]
fn splits_subcommand_doc_comment_between_short_and_long() {
    use clap::IntoApp;
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

    let mut app = LoremIpsum::into_app();

    let short_help = {
        let mut buffer = Vec::new();
        app.write_help(&mut buffer).ok();

        String::from_utf8(buffer).unwrap()
    };

    let long_help = {
        app.try_get_matches_from(vec!["test", "foo", "--help"])
            .expect_err("")
            .message
    };

    assert!(!short_help.contains("Or something else"));
    assert!(long_help.contains("DO NOT PASS A BAR UNDER ANY CIRCUMSTANCES"));
    assert!(long_help.contains("Or something else"));
}
