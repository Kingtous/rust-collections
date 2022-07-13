use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    os::unix::prelude::{AsRawFd, RawFd},
};

use bytes::{BufMut, BytesMut};
use libc::{epoll_create1, epoll_ctl, epoll_wait, fcntl};
use std::io;
use tokio_util::codec::Encoder;

pub struct RequestContext {
    stream: TcpStream,
    is_ok: bool,
}

fn epoll_create() -> io::Result<RawFd> {
    let fd = unsafe {
        let inner = epoll_create1(0);
        let flag = fcntl(inner, libc::F_GETFD);
        fcntl(inner, libc::F_SETFD, flag | libc::FD_CLOEXEC);
        inner
    };
    Ok(fd)
}

fn gen_http_response() -> BytesMut {
    let mut bc = tokio_util::codec::BytesCodec::new();
    let mut bytes = BytesMut::new();
    bytes.put(&b"hello world"[..]);

    let mut encoded = BytesMut::new();
    println!("origin: {:?}", bytes);
    let _ = bc.encode(bytes, &mut encoded);
    println!("now: {:?}", encoded);
    encoded
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:5000")?;
    let _ = listener.set_nonblocking(true);
    let fd = listener.as_raw_fd();
    // epoll
    let epoll_fd = epoll_create()?;
    // add tcp listener read event to epoll
    let mut key = 1000;
    let mut read_event = libc::epoll_event {
        events: (libc::EPOLLONESHOT | libc::EPOLLIN) as u32,
        u64: key,
    };
    unsafe {
        epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, fd, &mut read_event);
    }
    // open events loop
    let mut events: Vec<libc::epoll_event> = Vec::with_capacity(1024);
    let mut request_map = HashMap::<u64, RequestContext>::new();
    println!("initilized epoll model, listening to 127.0.0.1:5000");
    loop {
        events.clear();
        let res = unsafe {
            // epfd, events, maxevents, timeout
            epoll_wait(
                epoll_fd,
                events.as_mut_ptr() as *mut libc::epoll_event,
                1024,
                1000,
            )
        };
        if res < 0 {
            eprintln!("error with code {}", res);
            break;
        }
        // println!("read {} events.", res);
        let event_cnt = res as usize;
        unsafe {
            events.set_len(event_cnt);
        }
        for evt in &events {
            match evt.u64 {
                1000 => {
                    println!("recv one read request, start accept");
                    if let Ok((stream, _addr)) = listener.accept() {
                        let _ = stream.set_nonblocking(true)?;
                        key += 1;
                        unsafe {
                            // register read events for this stream
                            // epfd, op, fd, event
                            epoll_ctl(
                                epoll_fd,
                                libc::EPOLL_CTL_ADD,
                                stream.as_raw_fd(),
                                &mut libc::epoll_event {
                                    events: (libc::EPOLLONESHOT | libc::EPOLLIN) as u32,
                                    u64: key,
                                },
                            );
                            println!("registered stream with key {}", key);
                        }
                        request_map.insert(
                            key,
                            RequestContext {
                                stream,
                                is_ok: false,
                            },
                        );
                        // re-register listener epoll
                        unsafe {
                            epoll_ctl(
                                epoll_fd,
                                libc::EPOLL_CTL_MOD,
                                fd,
                                &mut libc::epoll_event {
                                    events: (libc::EPOLLONESHOT | libc::EPOLLIN) as u32,
                                    u64: 1000,
                                },
                            );
                        }
                    }
                }
                k => {
                    // handle tcp stream
                    match request_map.get_mut(&k) {
                        Some(request) => {
                            println!("processing request");
                            let mut stream = &request.stream;
                            let _ = evt.events;
                            let mut buf = vec![0; 1024];
                            match stream.read(&mut buf) {
                                Ok(sz) => {
                                    println!("receive {} bytes: {:?}", sz, buf);
                                }
                                Err(err) => {
                                    eprintln!("{}", err);
                                }
                            }
                            let resp = gen_http_response();
                            let _ = stream.write_all(&resp[..]);
                            request.is_ok = true;
                            println!("response completed.");
                        }
                        None => {
                            eprintln!("no such request stream {}", k);
                        }
                    }
                }
            }
        }
        // clean is_ok stream
        let cleaned: Vec<u64> = request_map
            .iter()
            .filter(|(_, v)| {
                return v.is_ok;
            })
            .map(|(key, _)| *key)
            .collect();
        for key in cleaned {
            request_map.remove_entry(&key);
        }
    }
    Ok(())
}
