/// 演示如何给一个函数注入多种功能
///
/// 使用trait完成下面两个函数作为一个函数的参数，即兼容以下用法：
/// 1. context.call(function_1);
/// 2. context.call(function_2);

fn function_1(id: u64) {
    println!("function 1 called: {}", id);
}

fn function_2(id: u64, duplicate_id: u64) {
    println!("function 2 called: {}, {}", id, duplicate_id);
}

pub struct Context {
    id: u64,
}

impl Context {
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    pub fn call<T, H>(&self, handler: H)
    where
        H: Handler<T>,
    {
        handler.call(self);
    }
}

pub trait FromContext {
    fn from_context(context: &Context) -> Self;
}

impl FromContext for u64 {
    fn from_context(context: &Context) -> Self {
        context.id
    }
}

pub trait Handler<T> {
    fn call(self, context: &Context);
}

/// 一个参数
impl<T1, F> Handler<T1> for F
where
    F: Fn(T1),
    T1: FromContext,
{
    fn call(self, context: &Context) {
        self(T1::from_context(context))
    }
}

impl<T1,T2, F> Handler<(T1,T2)> for F
where
    F: Fn(T1,T2),
    T1: FromContext,
    T2: FromContext,
{
    fn call(self, context: &Context) {
        self(T1::from_context(context), T2::from_context(context))
    }
}

fn main() {
    let context = Context::new(0);
    context.call(function_1);
    context.call(function_2);
}
