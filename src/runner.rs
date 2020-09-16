use std::io::{self};

pub trait Runner: Sized {
    fn run(&self) -> io::Result<std::process::Output>;
}