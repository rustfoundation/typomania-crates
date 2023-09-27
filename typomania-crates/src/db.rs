use std::collections::{HashMap, HashSet};

use chrono::Duration;
use futures::TryStreamExt;
use sqlx::{
    postgres::{types::PgInterval, PgPoolOptions},
    FromRow, PgPool,
};
use tracing::instrument;
use typomania::{AuthorSet, Package};

#[derive(Clone)]
pub(crate) struct Database(PgPool);

impl Database {
    #[instrument(level = "DEBUG", err)]
    pub(crate) async fn connect(url: &str) -> anyhow::Result<Self> {
        Ok(Self(PgPoolOptions::new().connect(url).await?))
    }

    #[instrument(level = "DEBUG", skip(self), err)]
    pub(crate) async fn get_most_popular_crates(
        &self,
        n: u32,
    ) -> anyhow::Result<HashMap<String, Krate>> {
        Ok(
            sqlx::query_file_as!(RawKrate, "queries/get_most_popular_crates.sql", n as i64)
                .fetch(&self.0)
                .map_ok(|krate| (krate.name.clone(), krate.into()))
                .try_collect()
                .await?,
        )
    }

    #[instrument(level = "DEBUG", skip(self), err)]
    pub(crate) async fn get_recently_updated_crates(
        &self,
        n: u32,
        since: Duration,
    ) -> anyhow::Result<HashMap<String, Krate>> {
        let interval = duration_to_interval(since)?;

        Ok(sqlx::query_file_as!(
            RawKrate,
            "queries/get_recent_crates.sql",
            n as i64,
            interval
        )
        .fetch(&self.0)
        .map_ok(|krate| (krate.name.clone(), krate.into()))
        .try_collect()
        .await?)
    }
}

#[derive(Debug, Clone, FromRow)]
#[allow(unused)]
struct RawKrate {
    pub(crate) name: String,
    pub(crate) login: Option<String>,
    pub(crate) homepage: Option<String>,
    pub(crate) repository: Option<String>,
    pub(crate) documentation: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) downloads: i64,
}

#[derive(Debug)]
pub(crate) struct Krate {
    pub(crate) authors: HashSet<String>,
    pub(crate) description: Option<String>,
}

impl From<RawKrate> for Krate {
    fn from(value: RawKrate) -> Self {
        Self {
            authors: if let Some(author) = value.login {
                [author].into_iter().collect()
            } else {
                HashSet::default()
            },
            description: value.description,
        }
    }
}

impl AuthorSet for Krate {
    fn contains(&self, author: &str) -> bool {
        self.authors.contains(author)
    }
}

impl Package for Krate {
    fn authors(&self) -> &dyn AuthorSet {
        self
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn shared_authors(&self, other: &dyn AuthorSet) -> bool {
        self.authors.iter().any(|author| other.contains(author))
    }
}

fn duration_to_interval(duration: Duration) -> anyhow::Result<PgInterval> {
    let days = duration.num_days();
    let fraction = duration - Duration::days(days);
    Ok(PgInterval {
        months: 0,
        days: days.try_into()?,
        microseconds: fraction
            .num_microseconds()
            .ok_or_else(|| anyhow::anyhow!("unexpected number of microseconds above 2^63"))?,
    })
}
