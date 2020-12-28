use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use std::convert::TryInto;
use std::io;

pub struct Setup;

impl Setup {

    pub fn new(data: Vec<u8>) -> io::Result<Self> {

        let ints: Vec<i32> = data.chunks_exact(4).map( |bytes| {
            i32::from_le_bytes(bytes.try_into().unwrap())
        }).collect();

        println!("Ints: {:#?}", ints);

        Ok(Self{})
    }

    pub fn from_file(path: &Path) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mut contents = Vec::<u8>::new(); 
        file.read_to_end(&mut contents)?;

        Setup::new(contents)
    }
}