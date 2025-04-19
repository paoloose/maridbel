macro_rules! static_assert {
    ($cond:expr, $msg:expr) => {
        #[allow(dead_code)]
        const fn static_assertion() {
            assert!($cond, $msg);
        }

        const _: () = static_assertion();
    };
    ($cond:expr) => {
        static_assert!($cond, "Static assertion failed");
    };
}

pub(crate) use static_assert;
