use crate::lib::Error;
use sqlx::{query, query_scalar, SqlitePool};

pub async fn get_deletion_password(
    pool: &SqlitePool,
    file_stem: &str,
    file_extension: &str,
) -> Result<Option<String>, Error> {
    query_scalar(
        r#"
            SELECT DELETION_PASSWORD
            FROM UPLOADS
            WHERE FILE_STEM = ?
            AND FILE_EXTENSION = ?
        "#,
    )
    .bind(file_stem)
    .bind(file_extension)
    .fetch_optional(pool)
    .await
    .map_err(Error::InternalServerError)
}

pub async fn insert_file(
    pool: &SqlitePool,
    file_stem: &str,
    file_extension: &str,
    deletion_password: &str,
) -> Result<bool, Error> {
    let res = query(
        r#"
            INSERT INTO UPLOADS
            VALUES (?, ?, ?)
        "#,
    )
    .bind(file_stem)
    .bind(file_extension)
    .bind(deletion_password)
    .execute(pool)
    .await;

    if let Some(err) = res.err() {
        if err.as_database_error().unwrap().code().unwrap() == "1555" {
            return Ok(false);
        }

        return Err(Error::INTERNAL_SERVER_ERROR);
    }

    Ok(true)
}

pub async fn delete_file(pool: &SqlitePool, file_stem: &str, file_extension: &str) -> Result<(), Error> {
    query(
        r#"
            DELETE FROM UPLOADS
            WHERE FILE_STEM = ?
            AND FILE_EXTENSION = ?
        "#,
    )
    .bind(file_stem)
    .bind(file_extension)
    .execute(pool)
    .await
    .map_err(Error::InternalServerError)?;

    Ok(())
}
