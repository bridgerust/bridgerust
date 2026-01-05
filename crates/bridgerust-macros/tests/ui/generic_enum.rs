#![allow(unexpected_cfgs)]
use bridgerust_macros::export;

#[export]
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}

