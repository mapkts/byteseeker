use crate::error::{Error, ErrorKind, Result};
use std::io::{Read, Seek, SeekFrom};

const DEFAULT_CHUNK_SIZE: usize = 1024;

/// Seeker that can seek the occurences of a given byte slice within a stream of bytes.
///
/// # Examples
///
/// ```
/// use byteseeker::ByteSeeker;
/// use std::io::Cursor;
///
/// let bytes = [b'0', b'\n', b'0'];
/// let mut cursor = Cursor::new(bytes);
/// let mut seeker = ByteSeeker::new(&mut cursor);
///
/// assert_eq!(seeker.seek(b"0").unwrap(), 0);
/// assert_eq!(seeker.seek_nth(b"0", 1).unwrap(), 2);
///
/// // After resetting, we can seek from the other direction.
/// seeker.reset();
/// assert_eq!(seeker.seek_back(b"0").unwrap(), 2);
/// assert_eq!(seeker.seek_nth_back(b"0", 1).unwrap(), 0);
/// ```
///
/// The `ByteSeeker` uses a internal buffer to read a chunk of bytes into memory to search the
/// occurences of a given byte slice. You can specify the capacity of the interal buffer by
/// initializing a `ByteSeeker` using `ByteSeeker::with_capacity`, if you are seeking within a
/// pretty small or pretty large stream. The default capacity of the internal buffer is default to
/// `1024` currently.
///
/// It's worth noting that seeking a byte slice whose length is greater than the `capacity` of the
/// calling `ByteSeeker` is not allowed.
#[derive(Debug)]
pub struct ByteSeeker<'a, RS: 'a + Read + Seek> {
    inner: &'a mut RS,
    buf: Vec<u8>,
    len: usize,
    cap: usize,
    state: State,
}

#[derive(Clone, Copy, Debug)]
struct State {
    lpos: usize,
    rpos: usize,
    done: bool,
    last: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum Dir {
    Start,
    End,
}

impl<'a, RS: 'a + Read + Seek> ByteSeeker<'a, RS> {
    /// Creates a new `ByteSeeker` that wraps a byte stream that implements `Read` and `Seek`.
    ///
    /// # Examples
    ///
    /// ```
    /// use byteseeker::ByteSeeker;
    /// use std::io::Cursor;
    ///
    /// let bytes = [1, 2, 3];
    /// let mut cursor = Cursor::new(bytes);
    /// let mut seeker = ByteSeeker::new(&mut cursor);
    /// ```
    pub fn new(stream: &'a mut RS) -> Self {
        ByteSeeker::with_capacity(stream, DEFAULT_CHUNK_SIZE)
    }

    /// Creates a new `ByteSeeker` that wraps a byte stream that implements `Read` and `Seek`, and
    /// sets the `capacity` of its internal buffer to the given capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use byteseeker::ByteSeeker;
    /// use std::io::Cursor;
    ///
    /// let bytes = [1, 2, 3];
    /// let mut cursor = Cursor::new(bytes);
    /// let mut seeker = ByteSeeker::with_capacity(&mut cursor, 3);
    /// ```
    pub fn with_capacity(stream: &'a mut RS, cap: usize) -> Self {
        // SAFETY: safe because `SeekFrom::End(0)` cannot return error.
        let len = stream.seek(SeekFrom::End(0)).unwrap() as usize;
        stream.seek(SeekFrom::Start(0)).unwrap();

        let rpos = if len == 0 { 0 } else { len - 1 };
        let state = State {
            lpos: 0,
            rpos: rpos,
            last: false,
            done: false,
        };

        Self {
            len,
            cap,
            state,
            inner: stream,
            buf: vecu8(DEFAULT_CHUNK_SIZE),
        }
    }

    /// Returns the length of the underlying byte stream.
    ///
    /// # Examples
    ///
    /// ```
    /// use byteseeker::ByteSeeker;
    /// use std::io::Cursor;
    ///
    /// let bytes = "lorem ipsum".as_bytes();
    /// let mut cursor = Cursor::new(bytes);
    /// let mut seeker = ByteSeeker::new(&mut cursor);
    ///
    /// assert_eq!(seeker.len(), 11);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the capacity of this `ByteSeeker`.
    ///
    /// # Examples
    ///
    /// ```
    /// use byteseeker::ByteSeeker;
    /// use std::io::Cursor;
    ///
    /// let bytes = "lorem ipsum".as_bytes();
    /// let mut cursor = Cursor::new(bytes);
    /// let mut seeker = ByteSeeker::with_capacity(&mut cursor, 11);
    ///
    /// assert_eq!(seeker.capacity(), 11);
    /// ```
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// Resets the state of the `ByteSeeker` to its original, so you can reuse this initialized
    /// `ByteSeeker` as it was newly created.
    ///
    /// # Examples
    ///
    /// ```
    /// use byteseeker::ByteSeeker;
    /// use std::io::Cursor;
    ///
    /// let bytes = [b'0', b'\n', b'0'];
    /// let mut cursor = Cursor::new(bytes);
    /// let mut seeker = ByteSeeker::new(&mut cursor);
    ///
    /// assert_eq!(seeker.seek(b"0").unwrap(), 0);
    /// assert_eq!(seeker.seek(b"0").unwrap(), 2);
    ///
    /// // The `reset` here is equivalent to:
    /// // let mut seeker = ByteSeeker::new(&mut cursor);
    /// seeker.reset();
    /// assert_eq!(seeker.seek_back(b"0").unwrap(), 2);
    /// assert_eq!(seeker.seek_back(b"0").unwrap(), 0);
    /// ```
    pub fn reset(&mut self) {
        self.inner.seek(SeekFrom::Start(0)).unwrap();

        let rpos = if self.len == 0 { 0 } else { self.len - 1 };
        self.state = State {
            lpos: 0,
            rpos: rpos,
            last: false,
            done: false,
        }
    }

    /// Searches for the given bytes **forwards**, and returns the offset (ralative to the start
    /// of the underlying byte stream) if the given bytes were found.
    ///
    /// If the initialized `ByteSeeker` haven't been called before, `seek` will start from
    /// the beginning; Otherwise, it will start from the last found `seek` position + 1.
    ///
    /// The `ByteSeeker` is stateful, which means you can call `seek` multiple times until
    /// reaching the end of the underlying byte stream.
    ///
    /// # Errors
    ///
    /// If the given bytes were not found, an error variant of `ErrorKind::ByteNotFound` will be
    /// returned. If any other I/O errors were encountered, an error variant of `ErrorKind::Io`
    /// will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use byteseeker::ByteSeeker;
    /// use std::io::Cursor;
    ///
    /// let bytes = [b'0', b'\n', b'\n', b'\n'];
    /// let mut cursor = Cursor::new(bytes);
    /// let mut seeker = ByteSeeker::new(&mut cursor);
    ///
    /// assert_eq!(seeker.seek(b"0").unwrap(), 0);
    /// assert_eq!(seeker.seek(b"\n\n").unwrap(), 1);
    /// assert_eq!(seeker.seek(b"\n\n").is_err(), true);
    /// ```
    pub fn seek(&mut self, bytes: &[u8]) -> Result<usize> {
        self.buf_seek(bytes, Dir::Start)
    }

    /// Searches for the given bytes **backwards**, and returns the offset (ralative to the start
    /// of the underlying byte stream) if the given bytes were found.
    ///
    /// If the initialized `ByteSeeker` haven't been called before, `seek` will start from
    /// the end; Otherwise, it will start from the last found `seek` position - 1.
    ///
    /// The `ByteSeeker` is stateful, which means you can call `seek_back` multiple times until
    /// reaching the end of the underlying byte stream.
    ///
    /// # Errors
    ///
    /// If the given bytes were not found, an error variant of `ErrorKind::ByteNotFound` will be
    /// returned. If any other I/O errors were encountered, an error variant of `ErrorKind::Io`
    /// will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use byteseeker::ByteSeeker;
    /// use std::io::Cursor;
    ///
    /// let bytes = [b'0', b'\n', b'\n', b'\n'];
    /// let mut cursor = Cursor::new(bytes);
    /// let mut seeker = ByteSeeker::new(&mut cursor);
    ///
    /// assert_eq!(seeker.seek_back(b"\n\n").unwrap(), 2);
    /// assert_eq!(seeker.seek_back(b"\n\n").is_err(), true);
    /// ```
    pub fn seek_back(&mut self, bytes: &[u8]) -> Result<usize> {
        self.buf_seek(bytes, Dir::End)
    }

    /// Seeks the nth occurence of the given bytes **forwards**, and returns the offset (ralative
    /// to the start of the underlying byte stream) if the given bytes were found.
    ///
    /// If the initialized `ByteSeeker` haven't been called before, `seek_nth`
    /// will start from the beginning; Otherwise, it will start from the last found `seek`
    /// position + 1.
    ///
    /// The `ByteSeeker` is stateful, which means you can call `seek_nth` multiple times until
    /// reaching the end of the underlying byte stream.
    ///
    /// # Errors
    ///
    /// If the given bytes were not found, an error variant of `ErrorKind::ByteNotFound` will be
    /// returned. If any other I/O errors were encountered, an error variant of `ErrorKind::Io`
    /// will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use byteseeker::ByteSeeker;
    ///
    /// let mut bytes = [b'\n', b'\n', b'\n', b'\n', b'\n'];
    /// let mut cursor = Cursor::new(bytes);
    /// let mut seeker = ByteSeeker::new(&mut cursor);
    /// assert_eq!(seeker.seek_nth(b"\n\n", 2).unwrap(), 2);
    /// assert_eq!(seeker.seek_nth(b"\n\n", 2).is_err(), true);
    /// ```
    pub fn seek_nth(&mut self, bytes: &[u8], nth: usize) -> Result<usize> {
        let mut counter = nth;
        loop {
            let pos = self.seek(bytes)?;
            counter -= 1;
            if counter == 0 {
                return Ok(pos);
            }
        }
    }

    /// Seeks the nth occurence of the given bytes **backwards**, and returns the offset (ralative
    /// to the start of the underlying byte stream) if the given bytes were found.
    ///
    /// If the initialized `ByteSeeker` haven't been called before, `seek_nth_back`
    /// will start from the end; Otherwise, it will start from the last found `seek` position - 1.
    ///
    /// The `ByteSeeker` is stateful, which means you can call `seek_nth_back` multiple times until
    /// reaching the end of the underlying byte stream.
    ///
    /// # Errors
    ///
    /// If the given bytes were not found, an error variant of `ErrorKind::ByteNotFound` will be
    /// returned. If any other I/O errors were encountered, an error variant of `ErrorKind::Io`
    /// will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use byteseeker::ByteSeeker;
    ///
    /// let mut bytes = [b'\n', b'\n', b'\n', b'\n', b'\n'];
    /// let mut cursor = Cursor::new(bytes);
    /// let mut seeker = ByteSeeker::new(&mut cursor);
    /// assert_eq!(seeker.seek_nth_back(b"\n\n", 2).unwrap(), 1);
    /// assert_eq!(seeker.seek_nth_back(b"\n\n", 2).is_err(), true);
    /// ```
    pub fn seek_nth_back(&mut self, bytes: &[u8], nth: usize) -> Result<usize> {
        let mut counter = nth;
        loop {
            let pos = self.seek_back(bytes)?;
            counter -= 1;
            if counter == 0 {
                return Ok(pos);
            }
        }
    }
}

impl<'a, RS: 'a + Read + Seek> ByteSeeker<'a, RS> {
    fn buf_seek(&mut self, bytes: &[u8], dir: Dir) -> Result<usize> {
        if self.state.done {
            return Err(Error::new(ErrorKind::ByteNotFound));
        }

        let bytes_len = bytes.len();
        if bytes_len == 0 || bytes_len > self.cap {
            return Err(Error::new(ErrorKind::UnsupportedLength));
        }

        use std::cmp::Ordering::*;
        match self.len.cmp(&bytes_len) {
            Less => {
                self.state.done = true;
                return Err(Error::new(ErrorKind::ByteNotFound));
            }
            Equal => {
                self.state.done = true;
                let mut buf = vecu8(bytes_len);
                self.inner.read_exact(&mut buf)?;
                if bytes == &buf {
                    return Ok(0);
                } else {
                    return Err(Error::new(ErrorKind::ByteNotFound));
                }
            }
            Greater => match dir {
                Dir::Start => loop {
                    if self.state.done {
                        return Err(Error::new(ErrorKind::ByteNotFound));
                    }

                    let remaining = self.len - self.state.lpos;
                    let mut buf_len = self.buf.len();

                    if remaining < bytes_len {
                        return Err(Error::new(ErrorKind::ByteNotFound));
                    } else if remaining < buf_len {
                        self.buf.truncate(remaining);
                        buf_len = remaining;
                        self.state.last = true;
                    }
                    self.inner.read_exact(&mut self.buf)?;

                    if let Some(pos) = self.buf.iter().position(|&x| x == bytes[0]) {
                        let cpos = self.state.lpos + pos;
                        if self.match_in_place(cpos, bytes)? {
                            if cpos + bytes_len > self.len {
                                self.state.done = true;
                            } else {
                                self.state.lpos = self
                                    .inner
                                    .seek(SeekFrom::Start((cpos + bytes_len) as u64))?
                                    as usize;
                            }
                            return Ok(cpos);
                        } else if buf_len == remaining {
                            return Err(Error::new(ErrorKind::ByteNotFound));
                        }
                    } else {
                        if self.state.last {
                            self.state.done = true;
                            return Err(Error::new(ErrorKind::ByteNotFound));
                        } else {
                            self.state.lpos = self
                                .inner
                                .seek(SeekFrom::Start((self.state.lpos + buf_len) as u64))?
                                as usize;
                        }
                    }
                },
                Dir::End => loop {
                    if self.state.done {
                        return Err(Error::new(ErrorKind::ByteNotFound));
                    }

                    let remaining = self.state.rpos + 1;
                    let mut buf_len = self.buf.len();

                    if remaining < bytes_len {
                        return Err(Error::new(ErrorKind::ByteNotFound));
                    } else if remaining < buf_len {
                        self.buf.truncate(remaining);
                        buf_len = remaining;
                        self.state.last = true;
                    }

                    self.inner
                        .seek(SeekFrom::Start((remaining - buf_len) as u64))?
                        as usize;
                    self.inner.read_exact(&mut self.buf)?;

                    if let Some(pos) = self
                        .buf
                        .iter()
                        .rev()
                        .position(|&x| x == bytes[bytes_len - 1])
                    {
                        let cpos = self.state.rpos - pos - (bytes_len - 1);
                        if self.match_in_place(cpos, bytes)? {
                            if (cpos as isize) - 1 < 0 {
                                self.state.done = true;
                            } else {
                                self.state.rpos =
                                    self.inner.seek(SeekFrom::Start((cpos - 1) as u64))? as usize;
                            }
                            return Ok(cpos);
                        } else if buf_len == remaining {
                            return Err(Error::new(ErrorKind::ByteNotFound));
                        }
                    } else {
                        if self.state.last {
                            self.state.done = true;
                            return Err(Error::new(ErrorKind::ByteNotFound));
                        } else {
                            self.state.rpos = remaining - buf_len;
                        }
                    }
                },
            },
        }
    }

    fn match_in_place(&mut self, pos: usize, bytes: &[u8]) -> Result<bool> {
        self.inner.seek(SeekFrom::Start(pos as u64))?;
        let len = bytes.len();
        if pos + len > self.len {
            return Ok(false);
        }
        let mut buf = vecu8(len);
        self.inner.read_exact(&mut buf)?;
        self.inner.seek(SeekFrom::Start(pos as u64))?;
        if &buf == bytes {
            return Ok(true);
        } else {
            return Ok(false);
        }
    }
}

// Creates a `Vec<u8>` whose capacity and length are exactly the same.
fn vecu8(len: usize) -> Vec<u8> {
    let mut vec = Vec::with_capacity(len);
    unsafe {
        vec.set_len(len);
    }
    vec
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::iter;

    #[test]
    fn test_vecu8() {
        let vec = vecu8(0);
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), 0);

        let vec = vecu8(42);
        assert_eq!(vec.len(), 42);
        assert_eq!(vec.capacity(), 42);
    }

    #[test]
    fn test_match_in_place() {
        let bytes: Vec<u8> = vec![0, 1, 2];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.match_in_place(0, &[0]).unwrap(), true);
        assert_eq!(seeker.match_in_place(0, &[0, 1]).unwrap(), true);
        assert_eq!(seeker.match_in_place(0, &[0, 1, 2]).unwrap(), true);
        assert_eq!(seeker.match_in_place(0, &[0, 1, 2, 3]).unwrap(), false);
        assert_eq!(seeker.match_in_place(1, &[1]).unwrap(), true);
        assert_eq!(seeker.match_in_place(1, &[1, 2]).unwrap(), true);
        assert_eq!(seeker.match_in_place(1, &[1, 2, 3]).unwrap(), false);
    }

    #[test]
    fn test_invalid_seeking_bytes() {
        let bytes: Vec<u8> = vec![0, 1, 2];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek(&[]) {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::UnsupportedLength => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = vec![0, 1, 2];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::with_capacity(&mut cursor, 3);
        let seeking_bytes = [0; 4];
        match seeker.seek(&seeking_bytes) {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::UnsupportedLength => assert!(true),
                _ => assert!(false),
            },
        }
    }

    #[test]
    fn test_seek_cs0() {
        let bytes: Vec<u8> = vec![];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek(b"\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }
        match seeker.seek(b"\r\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }
    }

    #[test]
    fn test_seek_back_cs0() {
        let bytes: Vec<u8> = vec![];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek_back(b"\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = vec![];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek_back(b"\r\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }
    }

    #[test]
    fn test_seek_cs1() {
        let bytes: Vec<u8> = vec![b'0'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek(b"\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = vec![b'0'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek(b"\r\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = vec![b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek(b"\n").unwrap(), 0);
        match seeker.seek(b"\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => {
                    assert!(false)
                }
            },
        }

        let bytes: Vec<u8> = vec![b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek(b"\n\r") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }
    }

    #[test]
    fn test_seek_back_cs1() {
        let bytes: Vec<u8> = vec![b'0'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek_back(b"\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = vec![b'0'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek_back(b"\r\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = vec![b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek_back(b"\n").unwrap(), 0);
        match seeker.seek(b"\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => {
                    assert!(false)
                }
            },
        }

        let bytes: Vec<u8> = vec![b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek_back(b"\n\r") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }
    }

    #[test]
    fn test_seek_cs2() {
        let bytes: Vec<u8> = vec![b'0', b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek(b"\n").unwrap(), 1);
        match seeker.seek(b"\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = vec![b'0', b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek(b"\n\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = vec![b'\n', b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek(b"\n").unwrap(), 0);
        assert_eq!(seeker.seek(b"\n").unwrap(), 1);
        match seeker.seek(b"\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = vec![b'\n', b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek(b"\n\n").unwrap(), 0);
        match seeker.seek(b"\n\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }
    }

    #[test]
    fn test_seek_back_cs2() {
        let bytes: Vec<u8> = vec![b'0', b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek_back(b"\n").unwrap(), 1);
        match seeker.seek_back(b"\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = vec![b'0', b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek_back(b"\n\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = vec![b'\n', b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek_back(b"\n").unwrap(), 1);
        assert_eq!(seeker.seek_back(b"\n").unwrap(), 0);
        match seeker.seek_back(b"\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = vec![b'\n', b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek_back(b"\n\n").unwrap(), 0);
        match seeker.seek_back(b"\n\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }
    }

    #[test]
    fn test_seek_cs3() {
        let bytes: Vec<u8> = vec![b'0', b'0', b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek(b"\n").unwrap(), 2);
        match seeker.seek(b"\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = vec![b'0', b'0', b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek(b"\n\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => {
                    assert!(false)
                }
            },
        }

        let bytes: Vec<u8> = vec![b'\n', b'0', b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek(b"\n").unwrap(), 0);
        assert_eq!(seeker.seek(b"\n").unwrap(), 2);
        match seeker.seek(b"\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = vec![b'\n', b'0', b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek(b"\n\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = vec![b'\n', b'\n', b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek(b"\n").unwrap(), 0);
        assert_eq!(seeker.seek(b"\n").unwrap(), 1);
        assert_eq!(seeker.seek(b"\n").unwrap(), 2);
        match seeker.seek(b"\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = vec![b'\n', b'\n', b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek(b"\n\n").unwrap(), 0);
        match seeker.seek(b"\n\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }
    }

    #[test]
    fn test_seek_back_cs3() {
        let bytes: Vec<u8> = vec![b'0', b'0', b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek_back(b"\n").unwrap(), 2);
        match seeker.seek_back(b"\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = vec![b'0', b'0', b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek_back(b"\n\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => {
                    assert!(false)
                }
            },
        }

        let bytes: Vec<u8> = vec![b'\n', b'0', b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek_back(b"\n").unwrap(), 2);
        assert_eq!(seeker.seek_back(b"\n").unwrap(), 0);
        match seeker.seek_back(b"\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = vec![b'\n', b'0', b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek_back(b"\n\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = vec![b'\n', b'\n', b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek_back(b"\n").unwrap(), 2);
        assert_eq!(seeker.seek_back(b"\n").unwrap(), 1);
        assert_eq!(seeker.seek_back(b"\n").unwrap(), 0);
        match seeker.seek_back(b"\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = vec![b'\n', b'\n', b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek_back(b"\n\n").unwrap(), 1);
        match seeker.seek_back(b"\n\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }
    }

    #[test]
    fn test_seek_csn() {
        let bytes: Vec<u8> = iter::repeat(0)
            .take(DEFAULT_CHUNK_SIZE - 1)
            .chain(iter::repeat(b'\n').take(2))
            .chain(iter::repeat(0).take(DEFAULT_CHUNK_SIZE))
            .chain(iter::repeat(b'\n').take(2))
            .collect();
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek(b"\n").unwrap(), DEFAULT_CHUNK_SIZE - 1);
        assert_eq!(seeker.seek(b"\n").unwrap(), DEFAULT_CHUNK_SIZE);
        assert_eq!(seeker.seek(b"\n").unwrap(), DEFAULT_CHUNK_SIZE * 2 + 1);
        assert_eq!(seeker.seek(b"\n").unwrap(), DEFAULT_CHUNK_SIZE * 2 + 2);
        match seeker.seek(b"\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = iter::repeat(0)
            .take(DEFAULT_CHUNK_SIZE - 1)
            .chain(iter::repeat(b'\n').take(2))
            .chain(iter::repeat(0).take(DEFAULT_CHUNK_SIZE))
            .chain(iter::repeat(b'\n').take(2))
            .collect();
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek(b"\n\n").unwrap(), DEFAULT_CHUNK_SIZE - 1);
        assert_eq!(seeker.seek(b"\n\n").unwrap(), DEFAULT_CHUNK_SIZE * 2 + 1);
        match seeker.seek(b"\n\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }
    }

    #[test]
    fn test_seek_back_csn() {
        let bytes: Vec<u8> = iter::repeat(0)
            .take(DEFAULT_CHUNK_SIZE - 1)
            .chain(iter::repeat(b'\n').take(2))
            .chain(iter::repeat(0).take(DEFAULT_CHUNK_SIZE))
            .chain(iter::repeat(b'\n').take(2))
            .collect();
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek_back(b"\n").unwrap(), DEFAULT_CHUNK_SIZE * 2 + 2);
        assert_eq!(seeker.seek_back(b"\n").unwrap(), DEFAULT_CHUNK_SIZE * 2 + 1);
        assert_eq!(seeker.seek_back(b"\n").unwrap(), DEFAULT_CHUNK_SIZE);
        assert_eq!(seeker.seek_back(b"\n").unwrap(), DEFAULT_CHUNK_SIZE - 1);
        match seeker.seek_back(b"\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let bytes: Vec<u8> = iter::repeat(0)
            .take(DEFAULT_CHUNK_SIZE - 1)
            .chain(iter::repeat(b'\n').take(2))
            .chain(iter::repeat(0).take(DEFAULT_CHUNK_SIZE))
            .chain(iter::repeat(b'\n').take(2))
            .collect();
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(
            seeker.seek_back(b"\n\n").unwrap(),
            DEFAULT_CHUNK_SIZE * 2 + 1
        );
        assert_eq!(seeker.seek_back(b"\n\n").unwrap(), DEFAULT_CHUNK_SIZE - 1);
        match seeker.seek_back(b"\n\n") {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }
    }

    #[test]
    fn test_seek_nth() {
        let bytes: Vec<u8> = iter::repeat(0)
            .take(DEFAULT_CHUNK_SIZE - 1)
            .chain(iter::repeat(b'\n').take(2))
            .chain(iter::repeat(0).take(DEFAULT_CHUNK_SIZE - 1))
            .chain(iter::repeat(b'\n').take(2))
            .chain(iter::repeat(0).take(100))
            .chain(iter::repeat(b'\n').take(2))
            .collect();

        let mut cursor = Cursor::new(bytes.clone());
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek_nth(b"\n", 1).unwrap(), DEFAULT_CHUNK_SIZE - 1);
        assert_eq!(seeker.seek_nth(b"\n", 1).unwrap(), DEFAULT_CHUNK_SIZE);
        assert_eq!(seeker.seek_nth(b"\n", 1).unwrap(), 2 * DEFAULT_CHUNK_SIZE);
        assert_eq!(
            seeker.seek_nth(b"\n", 1).unwrap(),
            2 * DEFAULT_CHUNK_SIZE + 1
        );
        assert_eq!(
            seeker.seek_nth(b"\n", 1).unwrap(),
            2 * DEFAULT_CHUNK_SIZE + 100 + 2
        );
        assert_eq!(
            seeker.seek_nth(b"\n", 1).unwrap(),
            2 * DEFAULT_CHUNK_SIZE + 100 + 3
        );
        match seeker.seek_nth(b"\n", 1) {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let mut cursor = Cursor::new(bytes.clone());
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek_nth(b"\n", 2).unwrap(), DEFAULT_CHUNK_SIZE);
        assert_eq!(
            seeker.seek_nth(b"\n", 2).unwrap(),
            2 * DEFAULT_CHUNK_SIZE + 1
        );
        assert_eq!(
            seeker.seek_nth(b"\n", 2).unwrap(),
            2 * DEFAULT_CHUNK_SIZE + 100 + 3
        );
        match seeker.seek_nth(b"\n", 1) {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let mut cursor = Cursor::new(bytes.clone());
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek_nth(b"\n", 1).unwrap(), DEFAULT_CHUNK_SIZE - 1);
        assert_eq!(seeker.seek_nth(b"\n", 2).unwrap(), 2 * DEFAULT_CHUNK_SIZE);
        assert_eq!(
            seeker.seek_nth(b"\n", 3).unwrap(),
            2 * DEFAULT_CHUNK_SIZE + 100 + 3
        );
        match seeker.seek_nth(b"\n", 1) {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let mut cursor = Cursor::new(bytes.clone());
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek_nth(b"\n\n", 1).unwrap(), DEFAULT_CHUNK_SIZE - 1);
        assert_eq!(seeker.seek_nth(b"\n\n", 1).unwrap(), 2 * DEFAULT_CHUNK_SIZE);
        assert_eq!(
            seeker.seek_nth(b"\n\n", 1).unwrap(),
            2 * DEFAULT_CHUNK_SIZE + 100 + 2
        );
        match seeker.seek_nth(b"\n\n", 1) {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let mut cursor = Cursor::new(bytes.clone());
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek_nth(b"\n\n", 2).unwrap(), 2 * DEFAULT_CHUNK_SIZE);
        match seeker.seek_nth(b"\n\n", 2) {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let mut cursor = Cursor::new(bytes.clone());
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek_nth(b"\n\n", 1).unwrap(), DEFAULT_CHUNK_SIZE - 1);
        assert_eq!(
            seeker.seek_nth(b"\n\n", 2).unwrap(),
            2 * DEFAULT_CHUNK_SIZE + 100 + 2
        );
        match seeker.seek_nth(b"\n\n", 1) {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }
    }

    #[test]
    fn test_seek_nth_back() {
        let bytes: Vec<u8> = iter::repeat(0)
            .take(DEFAULT_CHUNK_SIZE - 1)
            .chain(iter::repeat(b'\n').take(2))
            .chain(iter::repeat(0).take(DEFAULT_CHUNK_SIZE - 1))
            .chain(iter::repeat(b'\n').take(2))
            .chain(iter::repeat(0).take(100))
            .chain(iter::repeat(b'\n').take(2))
            .collect();

        let mut cursor = Cursor::new(bytes.clone());
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(
            seeker.seek_nth_back(b"\n", 1).unwrap(),
            2 * DEFAULT_CHUNK_SIZE + 100 + 3
        );
        assert_eq!(
            seeker.seek_nth_back(b"\n", 1).unwrap(),
            2 * DEFAULT_CHUNK_SIZE + 100 + 2
        );
        assert_eq!(
            seeker.seek_nth_back(b"\n", 1).unwrap(),
            2 * DEFAULT_CHUNK_SIZE + 1
        );
        assert_eq!(
            seeker.seek_nth_back(b"\n", 1).unwrap(),
            2 * DEFAULT_CHUNK_SIZE
        );
        assert_eq!(seeker.seek_nth_back(b"\n", 1).unwrap(), DEFAULT_CHUNK_SIZE);
        assert_eq!(
            seeker.seek_nth_back(b"\n", 1).unwrap(),
            DEFAULT_CHUNK_SIZE - 1
        );
        match seeker.seek_nth_back(b"\n", 1) {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let mut cursor = Cursor::new(bytes.clone());
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(
            seeker.seek_nth_back(b"\n", 2).unwrap(),
            2 * DEFAULT_CHUNK_SIZE + 100 + 2
        );
        assert_eq!(
            seeker.seek_nth_back(b"\n", 2).unwrap(),
            2 * DEFAULT_CHUNK_SIZE
        );
        assert_eq!(
            seeker.seek_nth_back(b"\n", 2).unwrap(),
            DEFAULT_CHUNK_SIZE - 1
        );
        match seeker.seek_nth_back(b"\n", 2) {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let mut cursor = Cursor::new(bytes.clone());
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(
            seeker.seek_nth_back(b"\n", 1).unwrap(),
            2 * DEFAULT_CHUNK_SIZE + 100 + 3
        );
        assert_eq!(
            seeker.seek_nth_back(b"\n", 2).unwrap(),
            2 * DEFAULT_CHUNK_SIZE + 1
        );
        assert_eq!(
            seeker.seek_nth_back(b"\n", 3).unwrap(),
            DEFAULT_CHUNK_SIZE - 1
        );
        match seeker.seek_nth_back(b"\n", 1) {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let mut cursor = Cursor::new(bytes.clone());
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(
            seeker.seek_nth_back(b"\n\n", 1).unwrap(),
            2 * DEFAULT_CHUNK_SIZE + 100 + 2
        );
        assert_eq!(
            seeker.seek_nth_back(b"\n\n", 1).unwrap(),
            2 * DEFAULT_CHUNK_SIZE
        );
        assert_eq!(
            seeker.seek_nth_back(b"\n\n", 1).unwrap(),
            DEFAULT_CHUNK_SIZE - 1
        );
        match seeker.seek_nth_back(b"\n\n", 1) {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let mut cursor = Cursor::new(bytes.clone());
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(
            seeker.seek_nth_back(b"\n\n", 2).unwrap(),
            2 * DEFAULT_CHUNK_SIZE
        );
        match seeker.seek_nth_back(b"\n\n", 2) {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }

        let mut cursor = Cursor::new(bytes.clone());
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(
            seeker.seek_nth_back(b"\n\n", 1).unwrap(),
            2 * DEFAULT_CHUNK_SIZE + 100 + 2
        );
        assert_eq!(
            seeker.seek_nth_back(b"\n\n", 2).unwrap(),
            DEFAULT_CHUNK_SIZE - 1
        );
        match seeker.seek_nth_back(b"\n\n", 1) {
            Ok(_) => assert!(false),
            Err(e) => match *e.kind() {
                ErrorKind::ByteNotFound => assert!(true),
                _ => assert!(false),
            },
        }
    }
}
