use std::env;
use std::fs;

pub use cpm8080::CPU;

pub use cpm8080::CPM;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file = fs::read(args[1].clone()).unwrap();
    let mut m = [0xfd;0x10000];
    m[0x100..file.len()+0x100].copy_from_slice(&file);
    m[5] = 8;
    let mut c = CPU::new();
    c.index();
    loop{
        c.next(&mut m);
    }
}
