use rocket::route::Route;

mod index;

pub fn route_factory() -> Vec<Route> {
    routes![index::index]
}