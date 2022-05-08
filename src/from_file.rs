use std::fs::File;
use std::io::prelude::*;
use log::info;

pub fn read(path: &str, dest: &mut [u8]){
    let mut fl = File::open(path).expect("File?");
    let val = fl.read(&mut dest[..]).expect("Read succ");
    info!("Read {} bytes", val);
}
