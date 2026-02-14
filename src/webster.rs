use regex::Regex;
use reqwest::blocking::{Client, Response};
use serde::de::{DeserializeOwned, IgnoredAny, SeqAccess, Visitor};
use serde::Deserialize;
use std::fmt;
use url::Url;

use super::english::{
    AntonymResponse, DefinitionResponse, Details, ExampleResponse, SynonymResponse,
};

pub struct WebsterApi {
    base_url: Url,
    collegiate_api_key: String,
    thesaurus_api_key: String,
    client: Client,
    regex: Regex,
}

impl WebsterApi {
    pub fn new(
        collegiate_api_key: impl Into<String>,
        thesaurus_api_key: impl Into<String>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            base_url: Url::parse("https://www.dictionaryapi.com/")?,
            collegiate_api_key: collegiate_api_key.into(),
            thesaurus_api_key: thesaurus_api_key.into(),
            client: Client::new(),
            regex: Regex::new(r"\{[^{}]*\}").unwrap(),
        })
    }

    fn get<T: DeserializeOwned>(
        &self,
        word: impl AsRef<str>,
        details: Details,
    ) -> anyhow::Result<T> {
        let (path, api_key) = match details {
            Details::Definitions | Details::Examples => (
                format!("api/v3/references/collegiate/json/{}", word.as_ref()),
                &self.collegiate_api_key,
            ),
            Details::Synonyms | Details::Antonyms => (
                format!("api/v3/references/thesaurus/json/{}", word.as_ref()),
                &self.thesaurus_api_key,
            ),
        };
        let mut url = self.base_url.join(&path)?;
        url.set_query(Some(&format!("key={}", api_key)));

        let response = self.client.get(url).send()?;

        self.handle_response(response)
    }

    pub fn get_definitions(&self, word: impl AsRef<str>) -> anyhow::Result<DefinitionResponse> {
        let resp: Vec<CollegiateEntry> = self.get(word, Details::Definitions)?;
        let entry = resp
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("empty response"))?;

        let CollegiateEntry {
            meta,
            def,
            shortdef,
        } = entry;

        let word = meta.id;
        let definitions = if !shortdef.is_empty() {
            shortdef
        } else {
            def.into_iter()
                .flat_map(|s| s.into_dts())
                .flat_map(|block| block.into_iter())
                .filter_map(|dt| {
                    if let DtElement::Text(s) = dt {
                        Some(s)
                    } else {
                        None
                    }
                })
                .filter_map(|s| self.clean_markup(s))
                .collect()
        };

        Ok(DefinitionResponse { word, definitions })
    }

    pub fn get_examples(&self, word: impl AsRef<str>) -> anyhow::Result<ExampleResponse> {
        let resp: Vec<CollegiateEntry> = self.get(word, Details::Examples)?;
        let entry = resp
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("empty response"))?;

        let CollegiateEntry {
            meta,
            def,
            shortdef: _,
        } = entry;

        let word = meta.id;
        let examples = def
            .into_iter()
            .flat_map(|s| s.into_dts())
            .flat_map(|block| block.into_iter())
            .filter_map(|dt| {
                if let DtElement::Vis(v) = dt {
                    Some(v.into_iter())
                } else {
                    None
                }
            })
            .flat_map(|v| v.into_iter().map(|x| x.t))
            .filter_map(|s| self.clean_markup(s))
            .collect();

        Ok(ExampleResponse { word, examples })
    }

    pub fn get_synonyms(&self, word: impl AsRef<str>) -> anyhow::Result<SynonymResponse> {
        let resp: Vec<ThesaurusEntry> = self.get(word, Details::Synonyms)?;
        let entry = resp
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("empty response"))?;

        let ThesaurusEntry { meta } = entry;
        let word = meta.id;
        let synonyms = meta.syns.into_iter().flatten().collect();

        Ok(SynonymResponse { word, synonyms })
    }

    pub fn get_antonyms(&self, word: impl AsRef<str>) -> anyhow::Result<AntonymResponse> {
        let resp: Vec<ThesaurusEntry> = self.get(word, Details::Antonyms)?;
        let entry = resp
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("empty response"))?;

        let ThesaurusEntry { meta } = entry;
        let word = meta.id;
        let antonyms = meta.ants.into_iter().flatten().collect();

        Ok(AntonymResponse { word, antonyms })
    }

    fn clean_markup(&self, s: String) -> Option<String> {
        let trimmed = self.regex.replace_all(&s, "").trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }

    fn handle_response<T: DeserializeOwned>(&self, response: Response) -> anyhow::Result<T> {
        let status = response.status();

        if status.is_success() {
            response.json().map_err(|e| e.into())
        } else {
            anyhow::bail!("HTTP error {} {}", status, response.text()?);
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CollegiateEntry {
    pub meta: CollegiateMeta,
    pub def: Vec<CollegiateDefSection>,
    pub shortdef: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CollegiateMeta {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct CollegiateDefSection {
    pub sseq: Vec<Vec<SenseOrSkip>>,
}

impl CollegiateDefSection {
    fn into_dts(self) -> impl Iterator<Item = Vec<DtElement>> {
        self.sseq
            .into_iter()
            .flat_map(|block| block.into_iter())
            .filter_map(|elt| match elt {
                SenseOrSkip::Sense { dt } => Some(dt),
                SenseOrSkip::Skip => None,
            })
    }
}

/// either parsed (when tag is "sense") or skipped
#[derive(Debug)]
pub enum SenseOrSkip {
    Sense { dt: Vec<DtElement> },
    Skip,
}

impl<'de> Deserialize<'de> for SenseOrSkip {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SensePayload {
            dt: Vec<DtElement>,
        }

        struct SenseOrSkipVisitor;
        impl<'de> Visitor<'de> for SenseOrSkipVisitor {
            type Value = SenseOrSkip;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a 2-element array [tag, object] for sseq element")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let tag: String = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::custom("missing tag"))?;
                if tag != "sense" {
                    let _ = seq.next_element::<IgnoredAny>()?;
                    return Ok(SenseOrSkip::Skip);
                }
                let payload: SensePayload = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::custom("missing sense object"))?;
                Ok(SenseOrSkip::Sense { dt: payload.dt })
            }
        }
        deserializer.deserialize_seq(SenseOrSkipVisitor)
    }
}

#[derive(Debug, Deserialize)]
pub struct Vis {
    pub t: String,
}

/// One item in a sense's `dt` array: ["text", s] or ["vis", arr].
#[derive(Debug)]
pub enum DtElement {
    Text(String),
    Vis(Vec<Vis>),
}

impl<'de> Deserialize<'de> for DtElement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DtElementVisitor;
        impl<'de> Visitor<'de> for DtElementVisitor {
            type Value = DtElement;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a 2-element array [tag, value] for dt element")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let tag: String = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::custom("missing tag"))?;
                match tag.as_str() {
                    "text" => {
                        let s: String = seq
                            .next_element()?
                            .ok_or_else(|| serde::de::Error::custom("missing text"))?;
                        Ok(DtElement::Text(s))
                    }
                    "vis" => {
                        let vis: Vec<Vis> = seq
                            .next_element()?
                            .ok_or_else(|| serde::de::Error::custom("missing vis"))?;
                        Ok(DtElement::Vis(vis))
                    }
                    _ => {
                        let _ = seq.next_element::<IgnoredAny>()?;
                        Ok(DtElement::Text(String::new()))
                    }
                }
            }
        }
        deserializer.deserialize_seq(DtElementVisitor)
    }
}

#[derive(Debug, Deserialize)]
pub struct ThesaurusEntry {
    pub meta: ThesaurusMeta,
    // pub def: Option<Vec<ThesaurusDefSection>>,
}

#[derive(Debug, Deserialize)]
pub struct ThesaurusMeta {
    pub id: String,
    pub syns: Vec<Vec<String>>,
    pub ants: Vec<Vec<String>>,
}
