use handlebars::handlebars_helper;

handlebars_helper!(contains: |x: Json, y: Json|{
    x.is_array() && x.as_array().unwrap().contains(y)
});
