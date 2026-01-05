#![allow(unexpected_cfgs)]
use bridgerust_macros::export;

#[export]
pub struct Container<T> {
    pub value: T,
}

