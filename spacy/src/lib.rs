//! This crate wraps spaCy to provide the single similarity operation we need to be able to run to
//! compare package descriptions from typomania.
//!
//! At runtime, this crate requires the Python `spacy` package to be available, along with whatever
//! language model is in use. For example, these `pyproject.toml` dependencies are sufficient to
//! use the `en_core_web_lg` model:
//!
//! ```toml
//! [tool.poetry.dependencies]
//! python = "^3.11"
//! spacy = "^3.6.1"
//! en-core-web-lg = {url = "https://github.com/explosion/spacy-models/releases/download/en_core_web_lg-3.6.0/en_core_web_lg-3.6.0-py3-none-any.whl"}
//! ```

use std::fmt::Debug;

use pyo3::{types::PyModule, IntoPy, Py, PyAny, PyErr, Python};
use thiserror::Error;
use tracing::instrument;

/// A language pipeline.
///
/// This corresponds to the [`Language` Python type][lang-py].
///
/// [lang-py]: https://spacy.io/api/language
pub struct Language(Py<PyAny>);

impl Debug for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = Python::with_gil(|py| -> Result<String, PyErr> {
            self.0.getattr(py, "lang")?.extract(py)
        })
        .unwrap_or_else(|_e| "unknown lang".into());

        f.debug_tuple("Language").field(&path).finish()
    }
}

impl Language {
    /// Loads a language pipeline.
    ///
    /// This corresponds to [`spacy.load()` in the spaCy API][spacy-load].
    ///
    /// [spacy-load]: https://spacy.io/api/top-level#spacy.load
    #[instrument(level = "TRACE", err)]
    pub fn load(model: &str) -> Result<Self, Error> {
        Python::with_gil(|py| {
            Ok(Self(
                PyModule::import(py, "spacy")?
                    .getattr("load")?
                    .call1((model,))?
                    .into(),
            ))
        })
    }

    /// Applies the language pipeline to the given string.
    ///
    /// This corresponds to [`Language.__call__()` in the spaCy API][spacy-call].
    ///
    /// [spacy-call]: https://spacy.io/api/language#call
    #[instrument(level = "TRACE", skip(input), err)]
    pub fn apply(&mut self, input: impl IntoPy<Py<PyAny>>) -> Result<Doc, Error> {
        Python::with_gil(|py| Ok(Doc(self.0.call1(py, (input.into_py(py),))?)))
    }
}

/// A spaCy [`Doc`][spacy-doc].
///
/// [spacy-doc]: https://spacy.io/api/doc
pub struct Doc(Py<PyAny>);

impl Debug for Doc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = Python::with_gil(|py| -> Result<String, PyErr> {
            self.0.getattr(py, "text")?.extract(py)
        })
        .unwrap_or_else(|_e| "unknown text".into());

        f.debug_tuple("Doc").field(&text).finish()
    }
}

impl Doc {
    /// Calculates the similarity between this document and another document.
    ///
    /// This corresponds to [`Doc.similarity()` in the spaCy API][doc-similarity].
    ///
    /// [doc-similarity]: https://spacy.io/api/doc#similarity
    #[instrument(level = "TRACE", err)]
    pub fn similarity(&mut self, other: &mut Self) -> Result<f64, Error> {
        Python::with_gil(|py| {
            Ok(self
                .0
                .getattr(py, "similarity")?
                .call1(py, (&other.0,))?
                .extract(py)?)
        })
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Python(#[from] pyo3::PyErr),
}
