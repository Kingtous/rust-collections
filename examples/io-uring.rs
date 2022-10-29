use std::{fs, os::fd::AsRawFd};

use io_uring::{opcode, types, IoUring};

const MAX_BUFF_LEN: u32 = 4096;
const USER_DATA: u64 = 100;

fn main() {
    let mut ring = IoUring::new(1024).unwrap();
    let f = fs::File::open("README.md").unwrap();
    let fd = f.as_raw_fd();
    // 使用io-uring读入buffer
    // 4kb
    let mut buf = [0; MAX_BUFF_LEN as usize];
    let read_e = opcode::Read::new(types::Fd(fd), buf.as_mut_ptr(), MAX_BUFF_LEN)
        .build()
        .user_data(USER_DATA);

    unsafe {
        ring.submission()
            .push(&read_e)
            .expect("ring submission failed");
    }
    // 等待这一个返回
    ring.submit_and_wait(1).unwrap();

    let cqe = ring.completion().next().unwrap();

    let ret = cqe.result();
    println!("read return {}", ret);

    unsafe {
        // 一定要ManuallyDrop
        let s = std::mem::ManuallyDrop::new(String::from_raw_parts(
            buf.as_mut_ptr(),
            ret as _,
            MAX_BUFF_LEN as _,
        ));
        println!("{}", s.as_str());
    }

    println!("io-uring read complete.")
}
