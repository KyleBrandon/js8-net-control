use rocket::route::Route;

mod index;

pub fn views_factory() -> Vec<Route> {
    let mut views: Vec<Route> = Vec::new();
    views.append(&mut index::route_factory());
    // append additional view factories here

    views
}    
