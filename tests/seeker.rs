use byteseeker::*;
use std::io::Cursor;
use std::iter;

const DEFAULT_CHUNK_SIZE: usize = 1024;

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
