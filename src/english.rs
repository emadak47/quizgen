use rand::prelude::*;
use serde::Deserialize;
use std::path::Path;

use crate::mcq::{Choice, Mcq};

#[derive(thiserror::Error, Debug)]
pub enum EnglishQuizError {
    #[error("API error")]
    ApiError(anyhow::Error),
    #[error("Data is invalid")]
    DataError,
    #[error("File error: {0}")]
    FileError(#[from] std::io::Error),
}

#[derive(Debug, Deserialize)]
pub struct DefinitionResponse {
    pub word: String,
    pub definitions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SynonymResponse {
    pub word: String,
    pub synonyms: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AntonymResponse {
    pub word: String,
    pub antonyms: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExampleResponse {
    pub word: String,
    pub examples: Vec<String>,
}

#[derive(Debug)]
pub enum Details {
    Definitions,
    Synonyms,
    Antonyms,
    Examples,
}

impl std::fmt::Display for Details {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Details::Definitions => write!(f, "definitions"),
            Details::Synonyms => write!(f, "synonyms"),
            Details::Antonyms => write!(f, "antonyms"),
            Details::Examples => write!(f, "examples"),
        }
    }
}

pub trait EnglishApi {
    fn get_definitions(&self, word: &str) -> anyhow::Result<DefinitionResponse>;
    fn get_examples(&self, word: &str) -> anyhow::Result<ExampleResponse>;
    fn get_synonyms(&self, word: &str) -> anyhow::Result<SynonymResponse>;
    fn get_antonyms(&self, word: &str) -> anyhow::Result<AntonymResponse>;
}

pub struct EnglishQuiz {
    apis: [Box<dyn EnglishApi>; 2],
    kind: Details,
    words: Vec<String>,
    selected: Vec<bool>,
}

impl EnglishQuiz {
    pub fn new(
        apis: [Box<dyn EnglishApi>; 2],
        source: &Path,
        kind: Details,
    ) -> Result<Self, EnglishQuizError> {
        let words: Vec<String> = std::fs::read_to_string(source)
            .map_err(EnglishQuizError::FileError)?
            .lines()
            .map(|line| line.trim().to_string())
            .collect();

        Ok(Self {
            apis,
            kind,
            selected: vec![false; words.len()],
            words,
        })
    }

    pub fn available_words(&self) -> usize {
        self.selected.iter().filter(|sel| !*sel).count()
    }

    pub fn select_word(&mut self) -> Result<&str, EnglishQuizError> {
        let index = self
            .selected
            .iter()
            .enumerate()
            .filter_map(|(i, &sel)| if !sel { Some(i) } else { None })
            .choose(&mut rand::rng())
            .ok_or(EnglishQuizError::DataError)?;

        self.selected[index] = true;

        Ok(&self.words[index])
    }

    fn generate_choices<const N: usize>(
        &self,
        word: &str,
        synonyms_resp: SynonymResponse,
    ) -> Result<[String; N], EnglishQuizError> {
        let mut synonyms: Vec<String> = synonyms_resp
            .synonyms
            .into_iter()
            .filter(|s| s.to_lowercase() != word.to_lowercase())
            .collect();
        synonyms.shuffle(&mut rand::rng());

        let mut choices: [String; N] = synonyms
            .into_iter()
            .take(N - 1)
            .chain([word.to_string()])
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| EnglishQuizError::DataError)?;
        choices.shuffle(&mut rand::rng());

        Ok(choices)
    }

    fn try_get<F, T>(&self, f: F) -> Result<T, EnglishQuizError>
    where
        F: Fn(&dyn EnglishApi) -> anyhow::Result<T>,
    {
        let mut last_err = None;
        for api in &self.apis {
            match f(api.as_ref()) {
                Ok(t) => return Ok(t),
                Err(e) => last_err = Some(e),
            }
        }
        Err(EnglishQuizError::ApiError(last_err.unwrap()))
    }

    pub fn generate_mcq<const N: usize>(&self, word: &str) -> Result<Mcq<N>, EnglishQuizError> {
        let synonyms_resp = self.try_get(|api| api.get_synonyms(word))?;

        let statement = match self.kind {
            Details::Synonyms => {
                let mut examples_resp = self.try_get(|api| api.get_examples(word))?;

                if examples_resp.examples.is_empty()
                    || synonyms_resp.synonyms.len() < N - 1
                    || synonyms_resp.word != examples_resp.word
                    || synonyms_resp.word != word
                {
                    return Err(EnglishQuizError::DataError);
                }

                let example_index = rand::rng().random_range(0..examples_resp.examples.len());
                std::mem::take(&mut examples_resp.examples[example_index])
            }
            Details::Definitions => {
                let mut definition_resp = self.try_get(|api| api.get_definitions(word))?;

                if definition_resp.definitions.is_empty()
                    || synonyms_resp.synonyms.len() < N - 1
                    || synonyms_resp.word != definition_resp.word
                    || synonyms_resp.word != word
                {
                    return Err(EnglishQuizError::DataError);
                }

                let definition_index =
                    rand::rng().random_range(0..definition_resp.definitions.len());
                std::mem::take(&mut definition_resp.definitions[definition_index])
            }
            _ => unimplemented!(),
        };

        let choices = self.generate_choices(word, synonyms_resp)?;
        let correct_index = choices
            .iter()
            .position(|c| c.to_lowercase() == word.to_lowercase())
            .expect("Correct choice is present");
        let solution = Choice::try_from(correct_index).expect("Choice is valid");

        Ok(Mcq::new(statement, choices, solution))
    }
}
