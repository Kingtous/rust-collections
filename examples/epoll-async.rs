use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    ptr::addr_of_mut,
    task::{Context, Poll},
};

use errno::errno;
use libc::{
    bind, epoll_create1, epoll_ctl, epoll_event, epoll_wait, sockaddr,
    sockaddr_in, socket, socklen_t, ssize_t, AF_INET, EAGAIN, EINTR, EPOLLIN, EPOLL_CTL_ADD,
    EPOLL_CTL_MOD, SOCK_STREAM,
};

type Task = Pin<Box<dyn Future<Output = ()>>>;
static mut TASK_ARRAY: Vec<Task> = Vec::new();
static mut NEW_TASK_STACK: Vec<usize> = Vec::new();
static mut EMPTY_STACK: Vec<usize> = Vec::new();
// 单线程，没有多线程读写问题。所以直接全局存储上一次的(fd, 触发方式)
static mut TASK_FD_OP: (i32, i32) = (0, 0);

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
}

struct Bind {
    fd: i32,
    addr: *mut sockaddr,
    len: socklen_t,
}

impl Bind {
    pub fn new(fd: i32, addr: *mut sockaddr, len: socklen_t) -> Self {
        Self { fd, addr, len }
    }
}

impl Future for Bind {
    type Output = Result<ssize_t, i32>;

    fn poll(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();

        let fd = unsafe { bind(this.fd, this.addr, this.len) };
        if fd != -1 && errno().0 == EAGAIN {
            unsafe {
                TASK_FD_OP = (fd, EPOLLIN);
            }
            return Poll::Pending;
        }

        return Poll::Ready(if fd > 0 { Ok(fd as _) } else { Err(errno().0) });
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
struct Executor {
    // fd, (index, event)
    fd_map: HashMap<i32, (usize, i32)>,
    epfd: i32,

    event_size: i32,
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
                epoll_ctl(self.epfd, EPOLL_CTL_ADD, TASK_FD_OP.0, addr_of_mut!(event));
            } else {
                EMPTY_STACK.push(index);
            }
        }
    }

    pub fn run(&mut self) {
        unsafe {
            // 循环获取
            let waker = futures::task::noop_waker();
            let mut cx = Context::from_waker(&waker);
            self.epfd = epoll_create1(0);
            loop {
                self.poll_new_task(&mut cx);

                let n = epoll_wait(self.epfd, self.events.as_mut_ptr(), 1024, -1);
                let errno = errno();
                if n == -1 && errno.0 == EINTR {
                    //中断
                    continue;
                }
                if n == 0 {
                    break;
                }
                // 开始处理
                self.handle_complete_task(&mut cx, n);
            }
        }
    }

    pub fn handle_complete_task(&mut self, _cx: &mut Context, size: i32) {
        unsafe {
            let size = size as usize;
            for index in 0..size {
                let fd = self.events[index].u64 as i32;

                let (index, event) = self.fd_map.get(&fd).unwrap().clone();
                // if TASK_ARRAY[index].as_mut().poll(cx).is_ready() {
                //     EMPTY_STACK
                // }
                if fd != TASK_FD_OP.0 {
                    // 不是当前的fd, 更新这条记录
                    let mut ev = epoll_event {
                        events: TASK_FD_OP.1 as u32,
                        u64: TASK_FD_OP.0 as u64,
                    };
                    epoll_ctl(self.epfd, EPOLL_CTL_ADD, TASK_FD_OP.0, addr_of_mut!(ev));

                    self.fd_map.insert(TASK_FD_OP.0, (index, TASK_FD_OP.1));
                }

                if event != TASK_FD_OP.1 {
                    let mut ev = epoll_event {
                        events: TASK_FD_OP.1 as u32,
                        u64: TASK_FD_OP.0 as u64,
                    };
                    epoll_ctl(self.epfd, EPOLL_CTL_MOD, TASK_FD_OP.0, addr_of_mut!(ev));

                    self.fd_map.get_mut(&fd).unwrap().1 = TASK_FD_OP.1;
                }
            }
        }
    }
}

fn main() {
    let mut executor = Executor::default();
    spawn(async {
        let socket = TcpSocket::new();
        let mut addr = unsafe {
            sockaddr_in {
                sin_family: AF_INET as u16,
                sin_port: 8080,
                sin_addr: libc::in_addr {
                    s_addr: u32::from_be_bytes([127, 0, 0, 1]).to_be(),
                },
                sin_zero: std::mem::zeroed(),
            }
        };
        let len = std::mem::size_of_val(&addr) as u32;
        let bind = Bind::new(
            socket.as_fd(),
            &mut addr as *mut libc::sockaddr_in as *mut libc::sockaddr,
            len,
        );
        let _ = bind.await;
    });
    executor.run();
}
