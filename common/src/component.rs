use async_std::sync::Arc;

pub trait HasComponent<C> {

    fn set(&mut self, component: Arc<C>);

    fn get(&self) -> Option<Arc<C>>;
}