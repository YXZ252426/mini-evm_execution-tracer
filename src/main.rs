struct Solution();

impl Solution {
    pub fn max_profit(prices: Vec<i32>) -> i32 {
        let mut min_prices = 10000;
        let mut max_profit = 0;
        for i in 0..prices.len() {
           if (prices[i] < min_prices) {
            min_prices = prices[i];
           }else {
            max_profit = max_profit.max(prices[i] - min_prices);
           } 
        }
        max_profit
    }
}
fn main() {
    for i in 0..5 {
        println!("{}", i);
    }
    println!("Hello, world!");
}
