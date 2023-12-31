use rocket::http::Status;
use rocket::Request;
use rocket_dyn_templates::{Template, context};

pub fn error_response(error_string: String) -> Status {
    if error_string.contains("Validation") || error_string.contains("400") {
        eprintln!("Validation Error: {}", error_string);
        return Status::BadRequest;
    }

    eprintln!("ERROR!: {}", error_string);
    return Status::InternalServerError;
}

#[macro_export]
macro_rules! no_shit {
    ($message:expr) => {
        $message.map_err(|err| crate::error::error_response(err.to_string()))?
    };
}

#[catch(404)]
pub fn not_found(_req: &Request) -> Template {
    Template::render("error", context! {
        error: "Thing Not Found! Whatever that thing you were looking for is, we didn't have one!"
    })
}

#[catch(400)]
pub fn you_done_fucked_up(_req: &Request) -> Template {
    Template::render("error", context! {
        error: "This is the error you see when <em>you</em> have screwed something up. That's right: this error is your fault somehow."
    })
}


#[catch(422)]
pub fn unprocessable(_req: &Request) -> Template {
    Template::render("error", context! {
        error: "Ho boy! You've just thrown some kind of data at me that I don't know how to deal with! and with that... I die. <em>bleh</em> "
    })
}

#[catch(500)]
pub fn server_error(_req: &Request) -> Template {
    Template::render("error", context! {
        error: "Aiieeeee! My hair is on fire! "
    })
}
