/// FFI
///
/// `cargo build --lib`
///
/// Python call [hello_world_ffi]
///
///
/// from ctypes import *
/// dll = ctypes.cdll
/// lib = dll.LoadLibrary("/home/kingtous/projects/rust-test/target/debug/librust_test.so")
/// lib.hello_world_ffi.restype = c_char_p
/// ret = lib.hello_world_ffi()
/// ret: str = ret.decode()
/// assert ret == "hello world from rust!"
///

#[no_mangle]
pub fn hello_world_ffi() -> &'static str {
    return "hello world from rust!";
}

pub fn _main() {
    println!("{}", hello_world_ffi());
}
