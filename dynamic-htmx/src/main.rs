use std::sync::Mutex;

use rocket::State;
use rocket_dyn_templates::{Template, context};

#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> Template {
    Template::render(
        "index", // index.html.tera
        context!{}
    )
}


// This is responsible for rendering the page of a particular color.
#[get("/color/<color>")]
fn color_block(color: &str) -> Template {
    Template::render(
        "color_block", // color_block.html.tera
        context!{
            color: color
        }
    )
}

// Set favorite color
#[post("/color/favorite/<color>")]
fn set_favorite_color(color: &str, current_favorite: &State<Mutex<FavoriteColor>>) -> Template {
    let mut fav_lock = current_favorite.lock().unwrap();
    fav_lock.set(color);
    drop(fav_lock);
    Template::render(
        "message", // message.html.tera
        context! {
            message: format!("Favorite color set to {}", color)
        }
    )
}

// Render favorite color)
#[get("/color/favorite")]
fn get_favorite_color(current_favorite: &State<Mutex<FavoriteColor>>) -> Template {
    let fav_lock = current_favorite.lock().unwrap();
    match &fav_lock.get() {
        Some(color) => Template::render(
            "color_block", // color_block.html.tera
            context! {
                color: color
            }
        ),
        None => Template::render(
            "message", // message.html.tera
            context! {
                message: "Favorite color not set yet."
            }
        ),
    }
}

// A simple class just to contain the favorite color & manage setting it.
struct FavoriteColor {
    color_name: Option<String>
}

impl FavoriteColor {
    pub fn set(&mut self, color: &str) {
        self.color_name = Some(String::from(color))
    }

    pub fn get(&self) -> Option<String> {
        self.color_name.clone()
    }
}

#[launch]
fn rocket() -> _ {
    let favorite_container = FavoriteColor {color_name: None};
    rocket::build().mount(
        "/", routes![
            index,
            color_block,
            set_favorite_color,
            get_favorite_color
        ]
        )
        .attach(Template::fairing())
        // Mutex is used because this object is modified async.
        // To avoid race conditions/simulatenous access, we use a Mutex lock.
        // (The rust compiler won't let you do this otherwise)
        .manage(Mutex::new(favorite_container))
}

#[cfg(test)]
mod test {
    /*
    Reference material for unit testing rocket specifically: 
    https://rocket.rs/guide/v0.5/testing/#testing
     */
    use super::rocket;
    use rocket::{local::blocking::Client, http::Status};
    
    #[test]
    fn test_index() {
        // Rendering the index page should work successfully.
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get(uri!("/")).dispatch();
        assert!(response.status() == Status::Ok)
    }

    #[test]
    fn test_render_color() {
        // Rendering a particular color should return 200, and the correct elements (e.g css, text on page) should be present in the response 
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get(uri!("/color/green")).dispatch();
        assert!(response.status() == Status::Ok);
        let body = response.into_string().unwrap();
        assert!(body.contains("background-color: green;"));
        assert!(body.contains("<em> green </em>"));
    }

    #[test]
    fn test_get_favorite_unset() {
        // Getting the favorite color when it is not set should return a 200 and note that the favorite color is not set yet.
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get(uri!("/color/favorite")).dispatch();
        assert!(response.status() == Status::Ok);
        let body = response.into_string().unwrap();
        assert!(body.contains("Favorite color not set yet."))
    }

        
    #[test]
    fn test_set_favorite() {
        // Setting the favorite color should work & return a 200 along with a message stating that it was set.
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.post(uri!("/color/favorite/red")).dispatch();
        assert!(response.status() == Status::Ok);
        let body = response.into_string().unwrap();
        assert!(body.contains("Favorite color set to red"))
    }

    #[test]
    fn test_set_and_get_favorite() {
        // Setting the favorite color, and then getting the favorite color, should return the favorite color.
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let set_response = client.post(uri!("/color/favorite/blue")).dispatch();
        assert!(set_response.status() == Status::Ok);
        let set_body = set_response.into_string().unwrap();
        assert!(set_body.contains("Favorite color set to blue"));

        // Now, we expect calling `/color/favorite` to give us blue.
        let get_response = client.get(uri!("/color/favorite")).dispatch();
        assert!(get_response.status() == Status::Ok);
        let get_body = get_response.into_string().unwrap();
        assert!(get_body.contains("background-color: blue;"));
        assert!(get_body.contains("<em> blue </em>"));
    }
}