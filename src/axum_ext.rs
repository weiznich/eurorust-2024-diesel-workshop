//! Module for axum extensions
//!
//! This currently only contains a custom header parser for the AcceptLanguage header
//!
//! This header is later used for localization
use axum::http::{header, HeaderValue};
use axum_extra::headers::Header;
use fluent_templates::LanguageIdentifier;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AcceptLanguageError {
    #[error("Invalid value")]
    InvalidValue,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AcceptedLanguage {
    pub value: String,
    pub quality: f32,
}

impl Eq for AcceptedLanguage {}

impl PartialEq for AcceptedLanguage {
    fn eq(&self, other: &Self) -> bool {
        self.quality == other.quality && self.value.eq(&other.value)
    }
}

impl PartialOrd for AcceptedLanguage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AcceptedLanguage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.quality > other.quality {
            std::cmp::Ordering::Greater
        } else if self.quality < other.quality {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

impl FromStr for AcceptedLanguage {
    type Err = AcceptLanguageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut value = s.trim().split(';');
        let (value, quality) = (value.next(), value.next());

        let Some(value) = value else {
            return Err(AcceptLanguageError::InvalidValue);
        };

        if value.is_empty() {
            return Err(AcceptLanguageError::InvalidValue);
        }

        let quality = if let Some(quality) = quality.and_then(|q| q.strip_prefix("q=")) {
            quality.parse::<f32>().unwrap_or(0.0)
        } else {
            1.0
        };

        Ok(AcceptedLanguage {
            value: value.to_string(),
            quality,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct AcceptLanguage(pub Vec<AcceptedLanguage>);

impl Default for AcceptLanguage {
    fn default() -> Self {
        Self(vec![AcceptedLanguage {
            value: "en-US".into(),
            quality: 1.0,
        }])
    }
}

impl Header for AcceptLanguage {
    fn name() -> &'static axum::http::HeaderName {
        &header::ACCEPT_LANGUAGE
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum_extra::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i axum::http::HeaderValue>,
    {
        let value = values
            .next()
            .ok_or_else(axum_extra::headers::Error::invalid)?;
        let str = value.to_str().expect("Accept-Language must be a string");
        let mut languages = str
            .split(',')
            .map(AcceptedLanguage::from_str)
            .collect::<Result<Vec<AcceptedLanguage>, AcceptLanguageError>>()
            .map_err(|_| axum_extra::headers::Error::invalid())?;

        languages.sort();

        Ok(AcceptLanguage(languages))
    }

    fn encode<E: Extend<axum::http::HeaderValue>>(&self, values: &mut E) {
        let val = self
            .0
            .iter()
            .map(|l| format!("{};q={}", l.value, l.quality))
            .collect::<Vec<_>>()
            .join(",");

        let val = HeaderValue::from_str(&val).expect("Accept-Language must be valid");

        values.extend(std::iter::once(val))
    }
}

impl AcceptLanguage {
    pub(crate) fn as_lang_ident_iter(&self) -> impl Iterator<Item = LanguageIdentifier> + '_ {
        self.0.iter().filter_map(|lang| lang.value.parse().ok())
    }
}
