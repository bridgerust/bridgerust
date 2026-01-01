#![allow(unexpected_cfgs)]
// This should fail - can't export private enum without pub
use bridgerust_macros::export;

#[export]
enum PrivateEnum {
    Variant1,
    Variant2,
}

