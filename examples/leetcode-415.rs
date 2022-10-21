use std::vec;

struct Solution;

impl Solution {
    pub fn remove_leading_zero(num: String) -> String {
        let mut index = 0;
        for ch in num.chars() {
            if ch == '0' {
                index += 1;
            } else {
                break;
            }
        }
        num.chars().skip(index).collect()
    }

    pub fn add_strings(num1: String, num2: String) -> String {
        // remove leading zero
        let num1 = Solution::remove_leading_zero(num1);
        let num2 = Solution::remove_leading_zero(num2);

        if num1.is_empty() && num2.is_empty() {
            return "0".to_string();
        }

        let mut buf: Vec<u8> = vec![];
        buf.reserve(std::cmp::max(num1.len(), num2.len()) + 1);
        // short long
        let long_one;
        let short_one = if num1.len() > num2.len() {
            long_one = num1;
            num2
        } else {
            long_one = num2;
            num1
        };

        let mut long_one_it = long_one.chars().rev();
        let mut sign: u8 = 0;
        for s_ch in short_one.chars().rev() {
            let l_ch = long_one_it.next().unwrap();

            let s_value = s_ch as u8 - '0' as u8;
            let l_value = l_ch as u8 - '0' as u8;
            let mut sum = s_value + l_value + sign;

            sign = 0;

            if sum >= 10 {
                sign = 1;
                sum -= 10;
            }
            buf.push(sum + '0' as u8);
        }

        while let Some(num) = long_one_it.next() {
            let mut sum = num as u8 - '0' as u8 + sign;
            sign = 0;

            if sum >= 10 {
                sign = 1;
                sum -= 10;
            }
            buf.push(sum + '0' as u8);
        }

        if sign == 1 {
            buf.push(1 + '0' as u8);
        }
        buf.reverse();
        String::from_utf8(buf).unwrap()
    }
}

fn main() {
    println!(
        "{}",
        Solution::add_strings("408".to_string(), "5".to_string())
    );
}
