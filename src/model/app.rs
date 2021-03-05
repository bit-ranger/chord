use handlebars::Handlebars;


pub trait AppContext{


    fn get_render(&self) -> &fn ();
}

// #[derive(Debug)]
// pub struct AppContextStruct<'reg> {
//
// }

// impl AppContextStruct {
//     const handlebars: Handlebars<'_> = Handlebars::new();
// }

// impl AppContext for AppContextStruct{
//
//
//
//     fn get_render(&self) -> &fn() {
//         Handlebars::render_template;
//     }
// }

