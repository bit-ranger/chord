pub trait HasComponent<C> {
    fn set(&mut self, component: async_std::sync::Arc<C>);

    fn get(&self) -> Option<async_std::sync::Arc<C>>;
}

#[macro_export]
macro_rules! pool {
    ($pn:ident, {$($cn:ident: $ct:ty),+}) => {
        #[derive(Default)]
        pub struct $pn {
            $(
                $cn: Option<async_std::sync::Arc<$ct>>,
            )*
        }

        $(
            impl crate::app::component::HasComponent<$ct> for $pn {
                fn set(&mut self, component: async_std::sync::Arc<$ct>) {
                    self.$cn = Some(component)
                }

                fn get(&self) -> Option<async_std::sync::Arc<$ct>> {
                    self.$cn.as_ref().map(|c| c.clone())
                }
            }
        )*

        static mut POOL: Option<$pn> = Option::None;

        impl $pn {

            fn pool_init() -> &'static mut $pn{
                unsafe {
                    POOL = Some($pn::default());
                    POOL.as_mut().unwrap()
                }
            }

            pub fn pool_ref() -> &'static $pn {
                unsafe { POOL.as_ref().unwrap() }
            }
        }
    };
}
