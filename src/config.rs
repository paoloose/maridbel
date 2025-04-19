use crate::macros::static_assert;

pub const PAGE_SIZE: usize = 4096;

static_assert!(PAGE_SIZE % 8 == 0);
