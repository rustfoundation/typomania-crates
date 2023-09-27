use std::{
    collections::HashMap,
    fmt::Display,
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        OnceLock,
    },
};

use clap::Parser;
use colored::Colorize;
use db::{Database, Krate};
use is_terminal::IsTerminal;
use supports_hyperlinks::supports_hyperlinks;
use tracing_subscriber::{
    fmt::{format::FmtSpan, time::Uptime},
    EnvFilter,
};
use typomania::{
    checks::{Bitflips, Omitted, SwappedWords, Typos},
    corpus, Corpus, Harness, Package,
};

#[cfg(feature = "nlp")]
use {spacy::Language, std::sync::Mutex};

mod config;
mod db;

use crate::config::Config;

static CRATE_NAME_ALPHABET: &str =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890-_";

#[derive(Parser)]
struct Opt {
    #[arg(short, long, default_value = "typomania.toml")]
    config: PathBuf,

    #[arg(short, long, env = "DATABASE_URL")]
    database_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_timer(Uptime::default())
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let opt = Opt::parse();
    let config = Config::load(&opt.config).await?;
    let db = Database::connect(&opt.database_url).await?;

    // Perform various startup tasks: getting spaCy initialised, and getting our crate corpora from
    // the database.
    #[cfg(feature = "nlp")]
    let (language, top_crates, new_crates) = futures::try_join!(
        init_spacy(),
        db.get_most_popular_crates(config.top_crates()),
        db.get_recently_updated_crates(config.top_crates(), config.new_crates_to_check()),
    )?;

    #[cfg(not(feature = "nlp"))]
    let (top_crates, new_crates) = futures::try_join!(
        db.get_most_popular_crates(config.top_crates()),
        db.get_recently_updated_crates(config.top_crates(), config.new_crates_to_check()),
    )?;

    let checker = Harness::builder()
        .with_check(Bitflips::new(
            CRATE_NAME_ALPHABET,
            top_crates.keys().map(|s| s.as_str()),
        ))
        .with_check(Omitted::new(CRATE_NAME_ALPHABET))
        .with_check(SwappedWords::new("-_"))
        .with_check(Typos::new(
            config.typos().iter().map(|(c, typos)| (*c, typos.clone())),
        ))
        .build(
            #[cfg(feature = "nlp")]
            TopCrates::new(&config, top_crates, language),
            #[cfg(not(feature = "nlp"))]
            TopCrates::new(&config, top_crates),
        );
    let results = tokio::task::spawn_blocking(move || {
        checker.check(new_crates.into_iter().map(|(name, krate)| {
            let package: Box<dyn Package> = Box::new(krate);
            (name, package)
        }))
    })
    .await??;

    let total = results.len();
    for (i, (name, squats)) in results.into_iter().enumerate() {
        if i != 0 {
            println!();
        } else {
            println!("Found {total} crates that merit investigation.");
            println!();
        }

        println!("{}:", KrateName(name));
        for squat in squats.into_iter() {
            println!("\t{}", squat);
        }
    }

    Ok(())
}

struct KrateName(String);

static SUPPORTS_HYPERLINKS: OnceLock<bool> = OnceLock::new();

impl Display for KrateName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = if *SUPPORTS_HYPERLINKS
            .get_or_init(|| std::io::stdout().is_terminal() && supports_hyperlinks())
        {
            format!(
                "{}",
                terminal_link::Link::new(&self.0, &format!("https://crates.io/crates/{}", self.0))
            )
        } else {
            self.0.clone()
        }
        .bold()
        .blue()
        .underline();

        write!(f, "{}", inner)
    }
}

#[cfg(feature = "nlp")]
async fn init_spacy() -> anyhow::Result<Language> {
    Ok(tokio::task::spawn_blocking(|| Language::load("en_core_web_lg")).await??)
}

struct TopCrates {
    crates: HashMap<String, Krate>,
    counters: Counters,
    config: Config,

    #[cfg(feature = "nlp")]
    language: Mutex<Language>,
}

impl TopCrates {
    #[cfg(not(feature = "nlp"))]
    fn new(config: &Config, crates: HashMap<String, Krate>) -> Self {
        Self {
            crates,
            counters: Counters::default(),
            config: config.clone(),
        }
    }

    #[cfg(feature = "nlp")]
    fn new(config: &Config, crates: HashMap<String, Krate>, language: Language) -> Self {
        Self {
            crates,
            counters: Counters::default(),
            config: config.clone(),
            language: Mutex::new(language),
        }
    }
}

impl Corpus for TopCrates {
    fn contains_name(&self, name: &str) -> typomania::Result<bool> {
        Ok(if self.crates.contains_key(name) {
            self.counters.found();
            true
        } else {
            self.counters.not_found();
            false
        })
    }

    fn get(&self, name: &str) -> typomania::Result<Option<&dyn Package>> {
        Ok(match self.crates.get(name) {
            Some(package) => {
                self.counters.found();
                Some(package)
            }
            None => {
                self.counters.not_found();
                None
            }
        })
    }

    fn possible_squat(
        &self,
        name_to_check: &str,
        package_name: &str,
        package: &dyn Package,
    ) -> typomania::Result<bool> {
        let mut is_possible_squat =
            corpus::default_possible_squat(self, name_to_check, package_name, package)?;

        #[cfg(feature = "nlp")]
        if is_possible_squat {
            if let Some(check_package) = self.get(name_to_check)? {
                if let (Some(check_desc), Some(package_desc)) =
                    (check_package.description(), package.description())
                {
                    let mut language = self.language.lock().map_err(|e| {
                        anyhow::anyhow!("error acquiring mutex on the language model: {e:?}")
                    })?;

                    let mut check_doc = language.apply(check_desc)?;
                    let mut package_doc = language.apply(package_desc)?;

                    if check_doc.similarity(&mut package_doc)? < self.config.similarity_threshold()
                    {
                        is_possible_squat = false;
                    }
                }
            }
        }

        if is_possible_squat && self.config.allowed(name_to_check, package_name) {
            is_possible_squat = false;
        }

        Ok(is_possible_squat)
    }
}

#[derive(Default)]
struct Counters {
    found: AtomicUsize,
    not_found: AtomicUsize,
}

impl Counters {
    fn found(&self) {
        self.found.fetch_add(1, Ordering::SeqCst);
    }

    fn not_found(&self) {
        self.not_found.fetch_add(1, Ordering::SeqCst);
    }
}

impl Drop for Counters {
    fn drop(&mut self) {
        tracing::debug!(
            found = self.found.load(Ordering::SeqCst),
            not_found = self.not_found.load(Ordering::SeqCst),
            "corpus usage complete"
        );
    }
}
