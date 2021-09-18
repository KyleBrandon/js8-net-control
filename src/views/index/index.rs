use rocket_dyn_templates::{Template};
use std::collections::HashMap;


#[get("/")]
pub fn index() -> Template {
    trace!(">>index");
    let context: HashMap<&str, &str> = [("name", "Jonathan")]
        .iter().cloned().collect();
    let t = Template::render("index", &context);

    trace!("<<index");
    return t;
}
