#![allow(unexpected_cfgs)]
// This should fail - tuple structs not supported
use bridgerust_macros::export;

#[export]
pub struct TupleStruct(i32, i32);

