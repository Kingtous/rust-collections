#![feature(macro_metavar_expr)]
#![feature(trace_macros)]

use std::any::type_name;

macro_rules! v_me_50 {
    () => {
        pub fn kfc() {
            println!("kfc!!!");
        }
    };
}

/// Example:
///
/// statements!(
/// struct Foo;
/// );
///
/// let foo = Foo;
macro_rules! statements {
    ($($stmt: stmt)*) => ($($stmt)*);
}

macro_rules! generate_type_wrapper_class {
    ($type: ty, $name: ident) => {
        struct $name {
            inner: $type,
        }

        impl $name {
            pub fn print_type(&self) {
                println!("this inner type is: {}", type_name::<$type>())
            }

            pub fn print_inner(&self) {
                println!("this inner value is: {}", self.inner)
            }
        }
    };
}

macro_rules! visibilities {
    //         ∨~~Note this comma, since we cannot repeat a `vis` fragment on its own
    ($($vis:vis,)*) => {};
}

visibilities! {
    , // no vis is fine, due to the implicit `?`
    pub,
    pub(crate),
    pub(in super),
    pub(in some_path),
}

// generate a wrapper class
generate_type_wrapper_class!(u32, U32Wrapper);

macro_rules! attach_iteration_counts {
    ( $( ( $( $inner:ident ),* ) ; )* ) => {
        ( $(
            $((
                stringify!($inner),
                ${index(1)}, // 外层循环次数
                ${index()}  // 当前层（默认为0层）循环次数
            ),)*
        )* )
    };
}

fn main() {
    // 函数注入
    trace_macros!(true);
    v_me_50!();
    kfc();
    concat!("Prefix", "123");
    let custom_u32 = U32Wrapper { inner: 0 };
    custom_u32.print_type();
    custom_u32.print_inner();

    // index
    let v = attach_iteration_counts! {
        ( hello ) ;
        ( indices , of ) ;
        () ;
        ( these, repetitions ) ;
    };
    println!("{:?}", v);
}
