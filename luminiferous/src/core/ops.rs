// below macros adapted from the rust stdlib
// https://github.com/rust-lang/rust/blob/master/library/core/src/internal_macros.rs

// implements `op &T` given `op T` is defined and T implements `Copy`
macro_rules! forward_ref_unop {
    (impl $imp:ident for $t:ty, $method:ident) => {
        impl $imp for &$t {
            type Output = <$t as $imp>::Output;

            #[inline]
            fn $method(self) -> <$t as $imp>::Output {
                $imp::$method(*self)
            }
        }
    };
}
pub(crate) use forward_ref_unop;

/// implements `&T op U`, `T op &U`, and `&T op &U` given `T op U` is defined and `T` and `U` impl `Copy`.
macro_rules! forward_ref_binop {
    (impl $imp:ident<$u:ty> for $t:ty, $method:ident) => {
        impl<'a> $imp<$u> for &'a $t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            fn $method(self, rhs: $u) -> <$t as $imp<$u>>::Output {
                $imp::$method(*self, rhs)
            }
        }

        impl<'a> $imp<&'a $u> for $t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            fn $method(self, rhs: &'a $u) -> <$t as $imp<$u>>::Output {
                $imp::$method(self, *rhs)
            }
        }

        impl<'a, 'b> $imp<&'a $u> for &'b $t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            fn $method(self, rhs: &'a $u) -> <$t as $imp<$u>>::Output {
                $imp::$method(*self, *rhs)
            }
        }
    };
}

pub(crate) use forward_ref_binop;

/// implements `T op= &U` given `T op= U` is defined and `U` impl `Copy`.
macro_rules! forward_ref_op_assign {
    (impl $imp:ident<$u:ty> for $t:ty, $method:ident) => {
        impl $imp<&$u> for $t {
            #[inline]
            fn $method(&mut self, rhs: &$u) {
                $imp::$method(self, *rhs);
            }
        }
    };
}
pub(crate) use forward_ref_op_assign;
