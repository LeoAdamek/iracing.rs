use iracing::setups;
use std::path::Path;
use std::env::args;

pub fn main() {
    let path = args().last().expect("Error: No Setup file specified");

    setups::Setup::from_file( Path::new(&path) );

}