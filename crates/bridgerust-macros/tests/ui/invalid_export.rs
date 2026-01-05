#![allow(unexpected_cfgs)]
// This should fail - can't export private function without pub
use bridgerust_macros::export;

#[export]
fn private_function() -> i32 {
    42
}

