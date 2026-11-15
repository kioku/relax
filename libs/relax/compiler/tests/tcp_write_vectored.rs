use std::io::{IoSlice, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

#[cfg(unix)]
#[test]
fn tcp_write_vectored_owned_and_borrowed() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut sender = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let (mut receiver, _) = listener.accept().unwrap();
    let bufs = [IoSlice::new(b""), IoSlice::new(b"abc"), IoSlice::new(b""), IoSlice::new(b"def")];

    // Small buffers fit in the empty socket. Check the write count, not how TCP
    // splits subsequent reads: stream reads are allowed to return partial data.
    assert_eq!(sender.write_vectored(&bufs).unwrap(), 6);
    assert_eq!((&sender).write_vectored(&bufs).unwrap(), 6);
    assert_eq!(sender.write_vectored(&[]).unwrap(), 0);
    assert_eq!((&sender).write_vectored(&[IoSlice::new(b"")]).unwrap(), 0);
    // More than IOV_MAX is a partial write, not an EINVAL error.
    let many = vec![IoSlice::new(b"x"); 1025];
    assert_eq!(sender.write_vectored(&many).unwrap(), 1024);
    sender.shutdown(Shutdown::Write).unwrap();
    assert!(sender.write_vectored(&bufs).is_err());
    let mut received = Vec::new();
    receiver.read_to_end(&mut received).unwrap();
    assert_eq!(&received[..12], b"abcdefabcdef");
    assert_eq!(&received[12..], vec![b'x'; 1024]);
}
