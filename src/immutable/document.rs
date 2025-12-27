pub trait Document: Send + Sync + Clone + 'static {
    fn empty() -> Self;
}
