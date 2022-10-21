use std::vec;

pub struct Solution;

impl Solution {
    pub fn four_sum(nums: Vec<i32>, target: i32) -> Vec<Vec<i32>> {
        if nums.len() < 4 {
            return vec![];
        }
        let mut nums = nums;
        nums.sort_by(|a, b| a.cmp(b));
        Solution::_four_sum(nums, target)
    }

    pub fn _four_sum(nums: Vec<i32>, target: i32) -> Vec<Vec<i32>> {
        let mut res: Vec<Vec<i32>> = vec![];
        let mut result: Vec<i32> = vec![];
        let mut key: Vec<String> = vec![];
        // clean nums
        let mut new_nums = vec![];

        let mut last_num = -1;
        let mut cnt = 0;
        for num in nums {
            if last_num == -1 || num != last_num {
                new_nums.push(num);
                last_num = num;
                cnt = 1;
                continue;
            }
            if cnt >= 4 {
                // no
                continue;
            }
            new_nums.push(num);
            cnt += 1;
        }
        println!("{:?}", new_nums);
        let index = new_nums.len() - 1;
        Solution::dfs(
            &new_nums,
            index,
            target as i64,
            &mut result,
            &mut res,
            &mut key,
        );
        res
    }

    // add nums[index]
    pub fn dfs(
        nums: &Vec<i32>,
        index: usize,
        mut target: i64,
        result: &mut Vec<i32>,
        output: &mut Vec<Vec<i32>>,
        keys: &mut Vec<String>,
    ) {
        if result.len() >= 4 && target != 0 {
            return;
        }
        let num = nums[index] as i64;
        if target - num > target {
            return;
        }
        result.push(nums[index]);
        target -= num;

        if target == 0 && result.len() == 4 {
            // ok
            let key = format!("{},{},{},{}", result[0], result[1], result[2], result[3]);
            if !keys.contains(&key) {
                output.push(result.clone());
                keys.push(key);
            }
            //println!("push result: {:?}", result);
            // 试试不放这个的结果
            if index != 0 {
                result.pop().unwrap();
                Solution::dfs(nums, index - 1, target + num, result, output, keys);
            } else {
                result.pop().unwrap();
            }
            return;
        }

        if index != 0 {
            Solution::dfs(nums, index - 1, target, result, output, keys);
            result.pop().unwrap();
            Solution::dfs(nums, index - 1, target + num, result, output, keys);
        } else {
            result.pop().unwrap();
        }
    }
}

pub fn main() {
    let nums = vec![1000000000, 1000000000, 1000000000, 1000000000];
    println!("four sum is: {:?}", Solution::four_sum(nums, -294967296));
}
