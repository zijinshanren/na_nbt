pub trait Document: Send + Sync + Clone + Never + 'static {}

pub trait Never {
    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn never() -> Self;
}

impl<T: Send + Sync + Clone + Never + 'static> Document for T {}
