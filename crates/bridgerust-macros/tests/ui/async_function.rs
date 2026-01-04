#![allow(unexpected_cfgs)]
// This should now compile - async functions are supported
use bridgerust_macros::export;

#[export]
pub async fn async_function() -> i32 {
    42
}

