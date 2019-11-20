pub trait EventHandler<M> {
    fn propagate(&self, _event: M) -> bool {
        true
    }
}
