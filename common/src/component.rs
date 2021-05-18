use async_std::sync::Arc;

pub trait HasComponent<C> {

    fn add(&mut self, name: &str, component: Arc<C>);

    fn get(&self, name: &str) -> Option<Arc<C>>;

    fn get_all(&self) -> Vec<(&str, Arc<C>)>;
}