use std::fmt::Display;

use rocket::{http::Status, response::status::Custom};

pub fn to_internal_server_err<D: Display>(err: D) -> Custom<String> {
    warn!("Internal Server Error: {err}");
    Custom(Status::InternalServerError, err.to_string())
}