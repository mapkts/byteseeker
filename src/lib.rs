//! This library provides the [`ByteSeeker`] type to seek the occurences of
//! a given byte slice ([`&[u8]`][byteslice]) from either end of a stream of bytes.
//!
//! The [`ByteSeeker`] is helpful if you want to check the presences of a certain byte slice, or
//! you want to read partial of contents when you know where to start.
//!
//! # Examples
//!
//! - Read the last line of a file, without loading the entire file into memory.
//!
//! ```no_run
//! use byteseeker::{ByteSeeker, Result};
//! use std::io::{Read, Seek, SeekFrom};
//! use std::fs::File;
//!
//! fn read_last_line(f: &mut File, buf: &mut Vec<u8>) -> Result<()> {
//!     let mut seeker = ByteSeeker::new(f);
//!     let pos = seeker.seek_back(b"\n")?;
//!     let starting = if pos == seeker.len() - 1 {
//!         // if file ends with a newline.
//!         seeker.seek_back(b"\n")? + 1
//!     } else {
//!         // if file doesn't end with a newline.
//!         pos + 1
//!     };
//!
//!     f.seek(SeekFrom::Start(starting as u64))?;
//!     f.read_to_end(buf)?;
//!     Ok(())
//! }
//!
//! fn main() -> Result<()> {
//!    let mut f = File::open("./data.csv")?;
//!    let mut buf = Vec::new();
//!    read_last_line(&mut f, &mut buf)?;
//!
//!    // For simplicity, we just assume the given file is UTF-8 valid and unwrap the result here.
//!    println!("{}", std::str::from_utf8(&buf).unwrap());
//!
//!    Ok(())
//! }
//!
//! ```
//!
//! [`ByteSeeker`]: struct.ByteSeeker.html
//! [byteslice]: https://doc.rust-lang.org/std/primitive.slice.html
#![deny(missing_docs)]

mod error;
pub use error::{Error, ErrorKind, Result};

mod seeker;
pub use seeker::ByteSeeker;
