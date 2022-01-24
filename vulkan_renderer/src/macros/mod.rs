#[macro_export]
macro_rules! impl_get {
    ($class: ty, $name: ident, $ret: ty) => {
        impl $class {
            pub fn $name(&self) -> $ret {
                self.$name
            }
        }
    };
}

#[macro_export]
macro_rules! impl_get_ref {
    ($class: ty, $name: ident, $ret: ty) => {
        impl $class {
            pub fn $name(&self) -> $ret {
                &self.$name
            }
        }
    };
}
