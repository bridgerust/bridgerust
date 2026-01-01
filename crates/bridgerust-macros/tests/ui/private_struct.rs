#![allow(unexpected_cfgs)]
// This should fail - can't export private struct without pub
use bridgerust_macros::export;

#[export]
struct PrivateStruct {
    field: i32,
}

