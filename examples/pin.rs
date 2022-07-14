use std::{pin::Pin, ptr::NonNull};

#[derive(Clone)]
struct A {
    path: String,
    // 自引用变量
    self_p: NonNull<String>,
}

impl A {}

fn print_self_p(a: A) {
    let p = NonNull::from(&a.path);
    println!("current path addr: {:?}", p);
    println!("current self addr: {:?}", a.self_p);
    if &p != &a.self_p {
        println!("string has moved without Pin!!!")
    }
}

fn print_self_p_pin(a: Pin<Box<A>>) {
    let p = NonNull::from(&a.path);
    println!("current path addr: {:?}", p);
    println!("current self addr: {:?}", a.self_p);
    if &p != &a.self_p {
        println!("string has moved with Pin??? Weird!")
    } else {
        println!("string has NOT moved with Pin!!! Yeah!")
    }
}

fn main() {
    let mut a = A {
        path: "123".to_string(),
        self_p: NonNull::dangling(),
    };

    let p = NonNull::from(&a.path);
    println!("origin path addr: {:?}", p);
    a.self_p = p;
    let b = a.clone();
    print_self_p(a);
    //
    println!("now let's try Pin");
    let mut boxed = Box::pin(b);
    {
        boxed.self_p = NonNull::from(&boxed.path);
    }
    print_self_p_pin(boxed);
}
