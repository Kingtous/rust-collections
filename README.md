# Rust Collections

This is a collections to record programming with Rust, a high memory-safe programming language. I'll show u how to use them meanwhile I'm learning it. Check `examples` folder to see examples.

Q: How to run examples?

A: `cargo run --example=xxx`

## Async IO

- tokio
    - epoll model
- monoio
    - using io-uring, thread-per-core model, supports only Linux now

## Network Stack&Framework

- hyper
- tower
- reqwest
    - http/tcp/udp request lib
- serde_json
    - json serialize/deserialize lib
- rustls
    - TLS in pure Rust

## Desktop

- rustdesk-core
    - provide tunnel connections
- rdev
    - listen&simulate lib for all platforms.

## FFI

- pure rust static/dynamic lib
    - yes, pure FFI. Suitable for all other languages which supports FFI.
- flutter_rust_bridge
    - generate binding for flutter, a UI framework developed by Google.