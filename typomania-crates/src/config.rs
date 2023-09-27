use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::Arc,
};

use chrono::Duration;
use serde::{de::Error, Deserialize, Deserializer};
use tokio::{fs::File, io::AsyncReadExt};
use tracing::instrument;

#[derive(Clone)]
pub(crate) struct Config(Arc<Inner>);

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Inner {
    #[cfg(feature = "nlp")]
    similarity_threshold: Option<f64>,

    #[serde(deserialize_with = "deser_duration")]
    new_crates_to_check: Duration,
    top_crates: Option<u32>,

    allowed_crates: HashMap<String, HashSet<String>>,
    typos: HashMap<char, Vec<String>>,
}

impl Config {
    #[instrument(level = "DEBUG", err)]
    pub(crate) async fn load(path: &Path) -> anyhow::Result<Self> {
        let mut buf = String::new();
        File::open(path).await?.read_to_string(&mut buf).await?;

        Ok(Self(toml::from_str(&buf)?))
    }

    pub(crate) fn allowed(&self, name: &str, target: &str) -> bool {
        self.0
            .allowed_crates
            .get(name)
            .map(|allowed_targets| allowed_targets.contains(target))
            .unwrap_or_default()
    }

    pub(crate) fn new_crates_to_check(&self) -> Duration {
        self.0.new_crates_to_check
    }

    #[cfg(feature = "nlp")]
    pub(crate) fn similarity_threshold(&self) -> f64 {
        self.0.similarity_threshold.unwrap_or(0.97)
    }

    pub(crate) fn top_crates(&self) -> u32 {
        self.0.top_crates.unwrap_or(3000)
    }

    pub(crate) fn typos(&self) -> &HashMap<char, Vec<String>> {
        &self.0.typos
    }
}

fn deser_duration<'de, D>(de: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    match parse_duration::parse(&String::deserialize(de)?) {
        Ok(std_dur) => Duration::from_std(std_dur).map_err(Error::custom),
        Err(e) => Err(Error::custom(e)),
    }
}
