#![allow(unexpected_cfgs)]
// This should fail - HashMap not directly supported
use bridgerust_macros::export;
use std::collections::HashMap;

#[export]
pub fn process_map(map: HashMap<String, i32>) -> HashMap<String, i32> {
    map
}

