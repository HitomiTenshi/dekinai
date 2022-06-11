use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use derive_more::{Display, Error};
use pbkdf2::{
    password_hash::{PasswordHasher, SaltString},
    Params, Pbkdf2,
};
use rand::{distributions::Alphanumeric, prelude::ThreadRng, Rng};
use std::{ffi::OsStr, path::Path};

#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
#[derive(Debug, Display, Error)]
pub enum Error {
    BAD_REQUEST,
    INTERNAL_SERVER_ERROR,
    NOT_FOUND,
    UNAUTHORIZED,
}

#[allow(non_snake_case)]
impl Error {
    pub fn BadRequest<T>(_: T) -> Self {
        Self::BAD_REQUEST
    }

    pub fn InternalServerError<T>(_: T) -> Self {
        Self::INTERNAL_SERVER_ERROR
    }

    pub fn Unauthorized<T>(_: T) -> Self {
        Self::UNAUTHORIZED
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::new(self.status_code())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            Self::BAD_REQUEST => StatusCode::BAD_REQUEST,
            Self::INTERNAL_SERVER_ERROR => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NOT_FOUND => StatusCode::NOT_FOUND,
            Self::UNAUTHORIZED => StatusCode::UNAUTHORIZED,
        }
    }
}

pub fn get_file_stem_with_extension(filename: &str) -> (&str, String) {
    let filename = Path::new(filename);
    let file_stem = filename.file_stem().and_then(OsStr::to_str).unwrap_or("");

    let file_extension = filename
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_lowercase();

    (file_stem, file_extension)
}

pub fn get_file_extension(filename: &str) -> String {
    let filename = Path::new(filename);

    filename
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_lowercase()
}

pub fn get_random_text(rng: &mut ThreadRng, length: usize) -> String {
    rng.sample_iter(Alphanumeric)
        .take(length)
        .map(char::from)
        .collect::<String>()
}

pub fn hash_password(rng: &mut ThreadRng, password: &str) -> String {
    Pbkdf2
        .hash_password_customized(
            password.as_bytes(),
            None,
            None,
            Params {
                output_length: 32,
                rounds: 10,
            },
            SaltString::generate(rng).as_salt(),
        )
        .unwrap()
        .to_string()
}
