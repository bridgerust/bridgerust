#![allow(unexpected_cfgs)]
use bridgerust_macros::export;

#[export]
pub struct Container {
    pub data: std::collections::HashMap<String, i32>,
}

