#[macro_export]
macro_rules! LOG_GROUP {
    ($name:expr) => {
        const LOG_GROUP: &str = $name
    };
    () => {
        const LOG_GROUP: &str = "vbox";
    };
}

#[macro_export]
macro_rules! AssertCompile {
    ($cond:expr) => {
        const _ASSERT_COMPILE: () = assert!($cond);
    };
}

#[macro_export]
macro_rules! DECLCALLBACK {
    ($($tt:tt)*) => {
        $($tt)*
    };
}

#[macro_export]
macro_rules! RT_NORETURN {
    ($($tt:tt)*) => {
        $($tt)*
    };
}

#[macro_export]
macro_rules! RT_INDEFINITE_WAIT {
    () => {
        usize::MAX
    };
}

#[macro_export]
macro_rules! RT_INDEFINITE_TIMEOUT {
    () => {
        usize::MAX
    };
}

#[macro_export]
macro_rules! RT_ALIGN_Z {
    ($a:expr, $b:expr) => {
        (($a) + ($b) - 1) & !($b - 1)
    };
}

#[macro_export]
macro_rules! RT_ELEMENTS {
    ($arr:expr) => {
        $arr.len()
    };
}

#[macro_export]
macro_rules! RT_ZERO {
    ($ty:ty) => {
        <$ty>::default()
    };
}

#[macro_export]
macro_rules! RT_BZERO {
    ($ptr:expr, $len:expr) => {
        unsafe { core::ptr::write_bytes($ptr, 0, $len) }
    };
}

#[macro_export]
macro_rules! RT_BIT {
    ($n:expr) => {
        1u32 << ($n)
    };
}

#[macro_export]
macro_rules! RT_F_ZEROBIT {
    ($n:expr) => {
        !(1u32 << ($n))
    };
}

#[macro_export]
macro_rules! RT_BIT64 {
    ($n:expr) => {
        1u64 << ($n)
    };
}

#[macro_export]
macro_rules! VBOX_STATUS {
    ($status:expr) => {
        $status as i32
    };
}

#[macro_export]
macro_rules! RT_OK {
    () => {
        0i32
    };
}
