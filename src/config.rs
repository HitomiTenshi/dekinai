use crate::util;
use clap::ArgMatches;
use rand::thread_rng;
use sqlx::{Sqlite, SqlitePool, migrate, migrate::MigrateDatabase as _, sqlite::SqlitePoolOptions};
use std::path::PathBuf;

#[derive(Clone)]
pub struct AppConfig {
    pub blacklist: Option<Vec<String>>,
    pub output: PathBuf,
    pub password_hash: Option<String>,
    pub port: Option<u16>,
}

#[allow(dead_code)]
pub struct ServerConfig {
    pub unix: Option<PathBuf>,
    database_uri: String,
}

impl From<&ArgMatches> for AppConfig {
    fn from(matches: &ArgMatches) -> Self {
        let port: Option<u16>;

        #[cfg(unix)]
        {
            port = if !matches.get_flag("disable-port") {
                matches.get_one::<u16>("port").copied()
            } else {
                None
            };
        };

        #[cfg(not(unix))]
        {
            port = matches.get_one::<u16>("port").copied();
        };

        Self {
            blacklist: matches
                .get_many::<String>("blacklist")
                .map(|values| values.map(|value| value.to_lowercase()).collect()),
            output: matches.get_one::<PathBuf>("output").unwrap().to_owned(),
            password_hash: matches
                .get_one::<String>("password")
                .map(|value| util::hash_password(&mut thread_rng(), value)),
            port,
        }
    }
}

impl From<&ArgMatches> for ServerConfig {
    fn from(matches: &ArgMatches) -> Self {
        Self {
            unix: matches.get_one::<PathBuf>("unix").cloned(),
            database_uri: format!(
                "sqlite://{}",
                matches
                    .get_one::<PathBuf>("database")
                    .unwrap()
                    .join("dekinai.sqlite")
                    .display()
            ),
        }
    }
}

impl ServerConfig {
    pub async fn open_database_pool(&self) -> Result<SqlitePool, sqlx::Error> {
        if !Sqlite::database_exists(&self.database_uri).await? {
            Sqlite::create_database(&self.database_uri).await?;
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(num_cpus::get() as u32)
            .min_connections(1)
            .connect(&self.database_uri)
            .await?;

        migrate!().run(&pool).await?;
        Ok(pool)
    }
}
