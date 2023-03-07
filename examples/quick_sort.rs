fn quick_sort(nums: &mut Vec<i32>, st: usize, ed: usize) -> &mut Vec<i32>{
    if nums.len() == 0 || nums.len() == 1 || st >= ed {
        return nums;
    }
    let base = st;
    let base_num = nums[base];
    let mut left = st;
    let mut right = ed;
    println!("{:?}", nums);
    while left < right {
        while nums[right] >= base_num && left < right {
            right -= 1;
        }
        nums.swap(left, right);
        while nums[left] <= base_num && left < right {
            left += 1;
        }
        nums.swap(left, right);
    }
    nums[left] = base_num;
    quick_sort(nums, st, left - 1);
    quick_sort(nums, left + 1, ed);
    nums
}

fn main() {
    let mut v = vec![5,10,19,-1,2];
    let ed = v.len() - 1;
    println!("sorted: {:?}", quick_sort(&mut v, 0, ed));
}