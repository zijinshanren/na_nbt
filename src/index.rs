mod private {
    pub trait Sealed {}
    impl Sealed for usize {}
    impl Sealed for str {}
    impl Sealed for String {}
    impl<T> Sealed for &T where T: ?Sized + Sealed {}
}

pub trait Index: private::Sealed {
    #[doc(hidden)]
    fn index_dispatch<'a, V, R>(
        &self,
        value: &'a V,
        n: impl FnOnce(&'a V, usize) -> R,
        s: impl FnOnce(&'a V, &str) -> R,
    ) -> R;

    #[doc(hidden)]
    fn index_dispatch_mut<'a, V, R>(
        &self,
        value: &'a mut V,
        n: impl FnOnce(&'a mut V, usize) -> R,
        s: impl FnOnce(&'a mut V, &str) -> R,
    ) -> R;
}

impl Index for usize {
    #[inline]
    fn index_dispatch<'a, V, R>(
        &self,
        value: &'a V,
        n: impl FnOnce(&'a V, usize) -> R,
        _: impl FnOnce(&'a V, &str) -> R,
    ) -> R {
        n(value, *self)
    }

    #[inline]
    fn index_dispatch_mut<'a, V, R>(
        &self,
        value: &'a mut V,
        n: impl FnOnce(&'a mut V, usize) -> R,
        _: impl FnOnce(&'a mut V, &str) -> R,
    ) -> R {
        n(value, *self)
    }
}

impl Index for str {
    #[inline]
    fn index_dispatch<'a, V, R>(
        &self,
        value: &'a V,
        _: impl FnOnce(&'a V, usize) -> R,
        s: impl FnOnce(&'a V, &str) -> R,
    ) -> R {
        s(value, self)
    }

    #[inline]
    fn index_dispatch_mut<'a, V, R>(
        &self,
        value: &'a mut V,
        _: impl FnOnce(&'a mut V, usize) -> R,
        s: impl FnOnce(&'a mut V, &str) -> R,
    ) -> R {
        s(value, self)
    }
}

impl Index for String {
    #[inline]
    fn index_dispatch<'a, V, R>(
        &self,
        value: &'a V,
        _: impl FnOnce(&'a V, usize) -> R,
        s: impl FnOnce(&'a V, &str) -> R,
    ) -> R {
        s(value, self.as_str())
    }

    #[inline]
    fn index_dispatch_mut<'a, V, R>(
        &self,
        value: &'a mut V,
        _: impl FnOnce(&'a mut V, usize) -> R,
        s: impl FnOnce(&'a mut V, &str) -> R,
    ) -> R {
        s(value, self.as_str())
    }
}

impl<T: ?Sized + Index> Index for &T {
    #[inline]
    fn index_dispatch<'a, V, R>(
        &self,
        value: &'a V,
        n: impl FnOnce(&'a V, usize) -> R,
        s: impl FnOnce(&'a V, &str) -> R,
    ) -> R {
        (**self).index_dispatch(value, n, s)
    }

    #[inline]
    fn index_dispatch_mut<'a, V, R>(
        &self,
        value: &'a mut V,
        n: impl FnOnce(&'a mut V, usize) -> R,
        s: impl FnOnce(&'a mut V, &str) -> R,
    ) -> R {
        (**self).index_dispatch_mut(value, n, s)
    }
}
