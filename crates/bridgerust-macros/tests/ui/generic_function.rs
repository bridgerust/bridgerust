#![allow(unexpected_cfgs)]
use bridgerust_macros::export;

#[export]
pub fn process<T>(item: T) -> T {
    item
}

