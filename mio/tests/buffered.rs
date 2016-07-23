extern crate futures;
extern crate futuremio;
extern crate env_logger;

use std::net::TcpStream;
use std::thread;
use std::io::{Read, Write};

use futures::Future;
use futures::stream::Stream;
use futures::io;

macro_rules! t {
    ($e:expr) => (match $e {
        Ok(e) => e,
        Err(e) => panic!("{} failed with {:?}", stringify!($e), e),
    })
}

#[test]
fn echo_server() {
    drop(env_logger::init());

    let mut l = t!(futuremio::Loop::new());
    let srv = l.handle().tcp_listen(&"127.0.0.1:0".parse().unwrap());
    let srv = t!(l.run(srv));
    let addr = t!(srv.local_addr());

    let msg = "foo bar baz";
    let t = thread::spawn(move || {
        let mut s = t!(TcpStream::connect(&addr));

        let t2 = thread::spawn(move || {
            let mut s = t!(TcpStream::connect(&addr));
            let mut b = Vec::new();
            t!(s.read_to_end(&mut b));
            b
        });

        let mut expected = Vec::<u8>::new();
        for _i in 0..1024 {
            expected.extend(msg.as_bytes());
            assert_eq!(t!(s.write(msg.as_bytes())), msg.len());
        }
        (expected, t2)
    });

    let clients = srv.incoming().take(2).map(|e| e.0).collect();
    let copied = clients.and_then(|clients| {
        let mut clients = clients.into_iter();
        let a = io::BufReader::new(clients.next().unwrap());
        let b = io::BufWriter::new(clients.next().unwrap());
        io::copy(a, b)
    });

    let amt = t!(l.run(copied));
    let (expected, t2) = t.join().unwrap();
    let actual = t2.join().unwrap();

    assert!(expected == actual);
    assert_eq!(amt, msg.len() as u64 * 1024);
}