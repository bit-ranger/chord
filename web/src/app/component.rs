pub trait HasComponent<T> {
    fn set(&mut self, component: async_std::sync::Arc<T>);

    fn get(&self) -> Option<async_std::sync::Arc<T>>;

    fn get_ref(&self) -> Option<&T>;
}

#[macro_export]
macro_rules! pool {
    ($($cn:ident: $ct:ty),+) => {
        #[derive(Default)]
        pub struct Container {
            $(
                $cn: Option<async_std::sync::Arc<$ct>>,
            )*
        }

        $(
            impl crate::app::component::HasComponent<$ct> for Container {
                fn set(&mut self, component: async_std::sync::Arc<$ct>) {
                    self.$cn = Some(component)
                }

                fn get(&self) -> Option<async_std::sync::Arc<$ct>> {
                    self.$cn.as_ref().map(|c| c.clone())
                }

                fn get_ref(&self) -> Option<& $ct> {
                    self.$cn.as_ref()
                }
            }
        )*

        static mut __pool: Option<Container> = Option::None;

        pub fn init_pool() {
            unsafe {
                __pool = Some(Container::default());
            }
        }

        pub fn get_pool() -> &'static Container {
            unsafe { __pool.as_ref().unwrap() }
        }

        pub fn mut_pool() -> &'static mut Container {
            unsafe { __pool.as_ref().unwrap() }
        }
    };
}

mod test {
    use crate::app::component::HasComponent;
    use crate::app::conf::ConfigImpl;
    use async_std::sync::Arc;
    pool!(config: crate::app::ConfigImpl);

    #[test]
    fn test() {
        init_pool();
        let config: Option<Arc<ConfigImpl>> = get_pool().get();
    }
}
