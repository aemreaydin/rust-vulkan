pub(crate) unsafe fn to_u8_slice<T: Sized>(data: &T) -> &[u8] {
    std::slice::from_raw_parts((data as *const T) as *const u8, std::mem::size_of::<T>())
}

pub trait U8Slice {
    fn as_u8_slice(&self) -> &[u8];
}

macro_rules! impl_u8_slice {
    ($class: ty) => {
        use crate::slice_utils::{to_u8_slice, U8Slice};
        impl U8Slice for $class {
            fn as_u8_slice(&self) -> &[u8] {
                unsafe { to_u8_slice(self) }
            }
        }
    };
}

pub(crate) use impl_u8_slice;
