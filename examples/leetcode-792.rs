use std::collections::VecDeque;

struct Solution;

impl Solution {
    pub fn num_matching_subseq(s: String, words: Vec<String>) -> i32 {
        let len = s.len();
        let base = 'a' as u8;
        let mut map: Vec<VecDeque<&str>> = Vec::new();
        map.resize(26, VecDeque::with_capacity(words.len()));
        let mut cnt = 0;
        for word in words.iter() {
            let first_ch = word.as_bytes()[0] - base;
            map[first_ch as usize].push_back(word);
        }
        for i in 0..len {
            let ch = s.as_bytes()[i] - base;
            let mut times = (&mut map[ch as usize]).len();
            let mut element;
            loop {
                if times == 0 {
                    break;
                }
                element = (&mut map[ch as usize]).pop_front();
                if let Some(w) = element {
                    if w.len() == 1 {
                        cnt += 1;
                        println!("{}", w);
                    } else {
                        let sub_w = &w[1..];
                        let sub_ch = sub_w.chars().next().unwrap() as u8 - base;
                        (&mut map[sub_ch as usize]).push_back(sub_w);
                    }
                    times -= 1;
                } else {
                    break;
                }
            }
        }
        cnt
    }
}

fn main() {
    println!(
        "{}",
        Solution::num_matching_subseq(
            "abbcde".to_string(),
            vec![
                "a".to_string(),
                "bb".to_string(),
                "acd".to_string(),
                "ace".to_string()
            ]
        )
    )
}
