sed -i '4,8d' crates/services/src/lib.rs
sed -i '3a #![allow(unexpected_cfgs)]' crates/services/src/lib.rs
