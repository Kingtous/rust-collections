use std::{collections::HashMap, hash::Hash};

fn main() {
    let mut map = HashMap::<String, i64>::new();
    let str1 = "hello".to_string();

    let str2 = "hello";
    map.insert(str1, 1);
    println!("{:?}", map.get(str2));
}