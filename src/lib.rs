#![allow(uncommon_codepoints)]
// #![warn(missing_docs)]
#![doc = include_str!("../README.md")]
use std::fs;

use log::debug;

pub mod necro;
pub mod parse;
pub mod scroll;
pub mod value;

use necro::Necromancer;
use scroll::Scroll;

/// The error type for this library.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An error occurred while trying to find the scroll.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// An error occurred while trying to unroll and read the scroll.
    #[error(transparent)]
    Parse(#[from] nom::error::Error<&'static str>),
}

/// Load the scroll from the given path and parse it.
pub fn parse(path: &str) -> Result<Scroll, Error> {
    let code: &'static str = Box::new(fs::read_to_string(path)?).leak();

    let scroll = parse::parse(&code)?;
    Ok(scroll)
}

/// Perform the necromancy ritual with the scroll at the given location.
pub fn summon(path: &str) -> Result<(), Error> {
    let scroll = parse(path)?;

    debug!("{:?}", &scroll);
    Necromancer::unroll(scroll).initiate();
    Ok(())
}
