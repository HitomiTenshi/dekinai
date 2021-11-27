use crate::{config::AppConfig, db, lib, lib::Error};
use actix_multipart::Multipart;
use actix_web::{get, http::HeaderMap, post, web, HttpRequest};
use async_recursion::async_recursion;
use futures_util::{StreamExt, TryStreamExt};
use pbkdf2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Pbkdf2,
};
use rand::{prelude::ThreadRng, thread_rng};
use sqlx::SqlitePool;
use std::path::PathBuf;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

#[post("/")]
pub async fn upload(req: HttpRequest, mut payload: Multipart) -> Result<web::Json<[String; 2]>, Error> {
    let config = req.app_data::<AppConfig>().unwrap();
    let headers = req.headers();

    if let Some(password_hash) = &config.password_hash {
        validate_password(
            password_hash,
            headers
                .get("x-api-key")
                .ok_or(Error::UNAUTHORIZED)?
                .to_str()
                .map_err(Error::InternalServerError)?,
        )?;
    }

    let base_url = get_base_url(config, headers)?;

    if let Some(mut field) = payload.try_next().await.map_err(Error::InternalServerError)? {
        let file_extension = &lib::get_file_extension(
            field
                .content_disposition()
                .get_filename()
                .ok_or(Error::BAD_REQUEST)?,
        );

        if let Some(blacklist) = &config.blacklist {
            if blacklist.contains(file_extension) {
                return Err(Error::BAD_REQUEST);
            }
        }

        let (mut file, file_path, filename, deletion_password) = create_random_file(
            config,
            &mut thread_rng(),
            req.app_data::<SqlitePool>().unwrap(),
            file_extension,
        )
        .await?;

        while let Some(chunk) = &field.next().await {
            match chunk {
                Ok(bytes) => file.write_all(bytes).await.map_err(Error::InternalServerError)?,
                _ => {
                    let _ = fs::remove_file(&file_path).await;
                    return Err(Error::INTERNAL_SERVER_ERROR);
                }
            }
        }

        Ok(web::Json([
            base_url.clone() + &filename,
            base_url + &filename + "/" + &deletion_password,
        ]))
    } else {
        Err(Error::BAD_REQUEST)
    }
}

#[get("/{file}/{deletion_password}")]
pub async fn delete(req: HttpRequest, path: web::Path<(String, String)>) -> Result<&'static str, Error> {
    let config = req.app_data::<AppConfig>().unwrap();
    let pool = req.app_data::<SqlitePool>().unwrap();
    let file_path = &config.output.join(&path.0);
    let (file_stem, file_extension) = lib::get_file_stem_with_extension(&path.0);

    if file_path.exists() {
        if let Some(deletion_password) = &db::get_deletion_password(pool, file_stem, &file_extension).await? {
            validate_password(deletion_password, &path.1)?;
            db::delete_file(pool, file_stem, &file_extension).await?;
            let _ = fs::remove_file(file_path).await;
            Ok("File has been deleted.")
        } else {
            Err(Error::UNAUTHORIZED)
        }
    } else {
        db::delete_file(pool, file_stem, &file_extension).await?;
        Err(Error::NOT_FOUND)
    }
}

fn validate_password(hash: &str, password: &str) -> Result<(), Error> {
    let hash = &PasswordHash::new(hash).map_err(Error::InternalServerError)?;

    Pbkdf2
        .verify_password(password.as_bytes(), hash)
        .map_err(Error::Unauthorized)
}

#[async_recursion(?Send)]
async fn create_random_file(
    config: &AppConfig,
    rng: &mut ThreadRng,
    pool: &SqlitePool,
    file_extension: &str,
) -> Result<(File, PathBuf, String, String), Error> {
    let file_stem = lib::get_random_text(rng, 8);
    let deletion_password = lib::get_random_text(rng, 24);

    let filename = if !file_extension.is_empty() {
        file_stem.to_owned() + "." + file_extension
    } else {
        file_stem.to_owned()
    };

    let file_path = config.output.join(&filename);

    if !file_path.exists() {
        if db::insert_file(
            pool,
            &file_stem,
            file_extension,
            &lib::hash_password(rng, &deletion_password),
        )
        .await?
        {
            return Ok((
                File::create(&file_path)
                    .await
                    .map_err(Error::InternalServerError)?,
                file_path,
                filename,
                deletion_password,
            ));
        } else {
            db::delete_file(pool, &file_stem, file_extension).await?;
        }
    }

    create_random_file(config, rng, pool, file_extension).await
}

fn get_base_url(config: &AppConfig, headers: &HeaderMap) -> Result<String, Error> {
    let mut base_url = String::with_capacity(128);

    base_url.push_str(if let Some(proto) = headers.get("x-forwarded-proto") {
        proto.to_str().map_err(Error::BadRequest)?
    } else {
        "http"
    });

    base_url.push_str("://");

    let host = if let Some(host) = headers.get("host") {
        host.to_str().map_err(Error::BadRequest)?
    } else {
        "localhost"
    };

    base_url.push_str(host);

    if host == "localhost" {
        if let Some(port) = &config.port {
            base_url.push(':');
            base_url.push_str(port);
        }
    }

    if let Some(path) = headers.get("x-forwarded-path") {
        base_url.push_str(path.to_str().map_err(Error::BadRequest)?);
    }

    if !base_url.ends_with('/') {
        base_url.push('/');
    }

    Ok(base_url)
}
