use std::io::{self};

pub trait Runner {
    fn run(&self) -> io::Result<std::process::Output>;
}