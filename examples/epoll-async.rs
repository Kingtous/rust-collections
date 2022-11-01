use std::{
    collections::HashMap,
    future::Future,
    io::{Cursor, Read},
    num::NonZeroUsize,
    pin::Pin,
    process::exit,
    ptr::addr_of_mut,
    sync::{Arc, RwLock},
    task::{Context, Poll},
};

use errno::errno;
use lazy_static::lazy_static;
use libc::{
    accept, bind, epoll_create1, epoll_ctl, epoll_event, epoll_wait, fcntl, listen, send,
    setsockopt, sockaddr_in, socket, write, AF_INET,
    EAGAIN, EINTR, EPOLLIN, EPOLL_CTL_ADD, EPOLL_CTL_MOD, EWOULDBLOCK, F_SETFL,
    O_NONBLOCK, SOCK_STREAM, SOL_SOCKET, SO_REUSEADDR, SO_REUSEPORT,
};

type Task = Pin<Box<dyn Future<Output = ()>>>;
static mut TASK_ARRAY: Vec<Task> = Vec::new();
static mut NEW_TASK_STACK: Vec<usize> = Vec::new();
static mut EMPTY_STACK: Vec<usize> = Vec::new();
// 单线程，没有多线程读写问题。所以直接全局存储上一次的(fd, 触发方式)
static mut TASK_FD_OP: (i32, i32) = (0, 0);
static BACKLOG_CNT: i32 = 128;

lazy_static! {
    pub static ref EXECUTOR: Arc<RwLock<Executor>> = Arc::new(RwLock::new(Executor::default()));
    pub static ref EP_FD: Arc<RwLock<i32>> = Arc::new(RwLock::new(-1));
}

fn main() {
    println!("current PID: {}", std::process::id());
    // 使用所有的核来运行
    let cpus = std::thread::available_parallelism()
        .unwrap_or(NonZeroUsize::new(1).unwrap())
        .get();
    for i in 0..cpus {
        println!("start server on core {}", i);
        spawn(start_server(8080));
    }
    EXECUTOR.write().unwrap().run();
}

struct TcpSocket {
    fd: i32,
}

impl TcpSocket {
    pub fn new() -> Self {
        unsafe {
            return Self {
                fd: socket(AF_INET, SOCK_STREAM, 0),
            };
        }
    }

    pub fn as_fd(&self) -> i32 {
        self.fd
    }

    // bind an address
    pub async fn bind(&self, addr: sockaddr_in) -> Result<(), i32> {
        let bind = Listen::new(self.as_fd(), addr);
        bind.await
    }

    pub async fn accept(&self) -> Result<Client, i32> {
        unsafe {
            let accept = AsyncAccept {
                fd: self.fd,
                client_addr: std::mem::zeroed(),
            };
            return accept.await;
        }
    }
}

// 传入async的函数
fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    unsafe {
        if let Some(index) = EMPTY_STACK.pop() {
            TASK_ARRAY[index] = Box::pin(future);
        } else {
            TASK_ARRAY.push(Box::pin(future));
            NEW_TASK_STACK.push(TASK_ARRAY.len() - 1);
        }
    }
}

#[derive(Default)]
pub struct Executor {
    // fd, (index, event)
    fd_map: HashMap<i32, (usize, i32)>,
    events: Vec<epoll_event>,
}

impl Executor {
    // 在循环里面poll一个新task
    pub unsafe fn poll_new_task(&mut self, context: &mut Context) {
        while let Some(index) = NEW_TASK_STACK.pop() {
            // 开始poll新加入的task，第index个
            let task = &mut TASK_ARRAY[index];

            if task.as_mut().poll(context).is_pending() {
                self.fd_map.insert(TASK_FD_OP.0, (index, TASK_FD_OP.1));
                // 定义一个event，添加监听
                // events: 触发方式
                // u64: fd
                let mut event = epoll_event {
                    events: TASK_FD_OP.1 as _,
                    u64: TASK_FD_OP.0 as _,
                };
                println!("fd_map update to {:?}", self.fd_map);
                epoll_ctl(
                    *EP_FD.read().unwrap(),
                    EPOLL_CTL_ADD,
                    TASK_FD_OP.0,
                    addr_of_mut!(event),
                );
            } else {
                EMPTY_STACK.push(index);
            }
        }
    }

    pub fn run(&mut self) {
        self.events.reserve(1024);
        unsafe {
            // 循环获取
            let waker = futures::task::noop_waker();
            let mut cx = Context::from_waker(&waker);
            *EP_FD.write().unwrap() = epoll_create1(0);
            loop {
                self.poll_new_task(&mut cx);
                if self.fd_map.is_empty() {
                    break;
                }
                let n = epoll_wait(*EP_FD.read().unwrap(), self.events.as_mut_ptr(), 1024, -1);
                let errno = errno();

                if n == -1 {
                    if errno.0 == EINTR {
                        // 打断点会有一次epoll
                        continue;
                    } else {
                        eprintln!("{}", errno);
                        exit(0);
                    }
                }
                if n == 0 {
                    break;
                }
                // 设置长度
                self.events.set_len(n as usize);
                // 开始处理
                self.handle_complete_task(&mut cx, n);
            }
        }
    }

    pub fn handle_complete_task(&mut self, cx: &mut Context, size: i32) {
        unsafe {
            let size = size as usize;
            for index in 0..size {
                let fd = self.events[index].u64 as i32;

                let (index, event) = self.fd_map.get(&fd).unwrap().clone();
                if TASK_ARRAY[index].as_mut().poll(cx).is_ready() {
                    EMPTY_STACK.push(index);
                }
                if fd != TASK_FD_OP.0 {
                    // 不是当前的fd, 更新这条记录
                    let mut ev = epoll_event {
                        events: TASK_FD_OP.1 as u32,
                        u64: TASK_FD_OP.0 as u64,
                    };
                    epoll_ctl(
                        *EP_FD.read().unwrap(),
                        EPOLL_CTL_ADD,
                        TASK_FD_OP.0,
                        addr_of_mut!(ev),
                    );

                    self.fd_map.insert(TASK_FD_OP.0, (index, TASK_FD_OP.1));
                    println!("fd_map update to {:?}", self.fd_map);
                } else if event != TASK_FD_OP.1 {
                    let mut ev = epoll_event {
                        events: TASK_FD_OP.1 as u32,
                        u64: TASK_FD_OP.0 as u64,
                    };
                    epoll_ctl(
                        *EP_FD.read().unwrap(),
                        EPOLL_CTL_MOD,
                        TASK_FD_OP.0,
                        addr_of_mut!(ev),
                    );

                    self.fd_map.get_mut(&fd).unwrap().1 = TASK_FD_OP.1;
                }
            }
        }
    }
}

async fn start_server(port: u16) {
    let socket = TcpSocket::new();
    let listen_addr = unsafe {
        sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: port.to_be(),
            sin_addr: libc::in_addr {
                s_addr: u32::from_be_bytes([127, 0, 0, 1]).to_be(),
            },
            sin_zero: std::mem::zeroed(),
        }
    };
    match socket.bind(listen_addr).await {
        Ok(_) => {
            // accept
            loop {
                match socket.accept().await {
                    Ok(client) => {
                        println!("accept a client from fd={}", socket.as_fd());
                        client
                            .send_and_flush(Cursor::new("hello epoll async example"))
                            .await;
                    }
                    Err(_) => {
                        eprintln!("error while accept socket: {}", errno())
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("returned error {}", err);
        }
    }
}

struct Listen {
    fd: i32,
    addr: sockaddr_in,
}

impl Listen {
    pub fn new(fd: i32, addr: sockaddr_in) -> Self {
        Self { fd, addr }
    }
}

/// 实际上bind并不需要进行await，这里实现只是为了实现而实现
impl Future for Listen {
    type Output = Result<(), i32>;

    fn poll(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();
        let option_value = 1 as i32;
        // 设置非阻塞
        unsafe {
            let ret = fcntl(this.fd, F_SETFL, O_NONBLOCK);
            if ret != 0 {
                return Poll::Ready(Err(errno().0));
            }
        };
        // 设置重用
        let mut ret;
        unsafe {
            ret = setsockopt(
                this.fd,
                SOL_SOCKET,
                SO_REUSEADDR | SO_REUSEPORT,
                &option_value as *const i32 as _,
                std::mem::size_of_val(&option_value) as _,
            );
        }
        if ret != 0 {
            return Poll::Ready(Err(errno().0));
        }
        ret = unsafe {
            bind(
                this.fd,
                addr_of_mut!(this.addr) as _,
                std::mem::size_of_val(&this.addr) as _,
            )
        };
        if ret != 0 {
            return Poll::Ready(Err(errno().0));
        }
        ret = unsafe { listen(this.fd, BACKLOG_CNT) }; // backlog允许多少个等待连接，大于128个后会拒绝连接
        if ret == 0 {
            return Poll::Ready(Ok(()));
        }
        return Poll::Ready(Err(errno().0));
    }
}

/// 客户端
pub struct Client {
    client_fd: i32,
}

impl Client {
    // 一个假的async send
    pub async fn send_and_flush<IO: Read>(&self, mut data: IO) {
        let mut buf = Vec::<u8>::new();
        let _ = data.read_to_end(&mut buf);
        unsafe {
            send(self.client_fd, buf.as_ptr() as _, buf.len() as _, 0 as _);
        }
    }
}

/// Accept
struct AsyncAccept {
    fd: i32,

    client_addr: sockaddr_in,
}

impl Future for AsyncAccept {
    type Output = Result<Client, i32>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let mut addr_len = std::mem::size_of_val(&this.client_addr);
        let accept_fd;
        unsafe {
            accept_fd = accept(
                this.fd,
                addr_of_mut!(this.client_addr) as _,
                addr_of_mut!(addr_len) as _,
            );
            let err = errno();

            if err.0 == EAGAIN || err.0 == EWOULDBLOCK {
                TASK_FD_OP = (this.fd, EPOLLIN as _);
                // let epoll notify us
                let mut event = epoll_event {
                    events: EPOLLIN as _,
                    u64: this.fd as _, // 一个标识key, 我们直接使用fd
                };
                let _ret = epoll_ctl(
                    *EP_FD.read().unwrap(),
                    EPOLL_CTL_ADD,
                    this.fd,
                    &mut event as *mut epoll_event,
                );
                return Poll::Pending;
            }
            if accept_fd == -1 {
                return Poll::Ready(Err(err.0));
            }
            Poll::Ready(Ok(Client {
                client_fd: accept_fd,
            }))
        }
    }
}

/// Send

struct AsyncWrite {
    fd: i32,
    data: Vec<u8>,
}

impl Future for AsyncWrite {
    type Output = Result<isize, i32>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let ret;
        unsafe {
            ret = write(this.fd, this.data.as_ptr() as _, this.data.len());
        }
        if ret == -1 {
            let err = errno().0;
            if err == EAGAIN || err == EWOULDBLOCK {
                return Poll::Pending;
            }
        }
        return Poll::Ready(Ok(ret));
    }
}
