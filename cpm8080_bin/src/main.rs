#![feature(const_mut_refs)]
use cpm8080_lib::*;
use std::env;
use std::fs;

fn main() {
    simple_logger::init_with_level(log::Level::Trace).unwrap();
    let args: Vec<String> = env::args().collect();
    let file = fs::read(args[1].clone()).unwrap();
    let mut sys = Sys::new(&file);
    loop {
        sys.run_instruction();
    }
}
