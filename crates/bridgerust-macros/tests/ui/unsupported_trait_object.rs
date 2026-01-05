#![allow(unexpected_cfgs)]
// This should fail - trait objects not supported
use bridgerust_macros::export;

#[export]
pub fn process_trait(obj: Box<dyn std::fmt::Display>) -> String {
    obj.to_string()
}

