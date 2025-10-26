use rand::prelude::*;
use std::path::Path;

use crate::{
    mcq::{Choice, Mcq},
    words_api::{Details, SynonymResponse, WordsApi},
};

#[derive(thiserror::Error, Debug)]
pub enum EnglishQuizError {
    #[error("API error")]
    ApiError(anyhow::Error),
    #[error("Data is invalid")]
    DataError,
    #[error("File error: {0}")]
    FileError(#[from] std::io::Error),
}

pub struct EnglishQuiz {
    api: WordsApi,
    kind: Details,
    words: Vec<String>,
    selected: Vec<bool>,
}

impl EnglishQuiz {
    pub fn new(api: WordsApi, source: &Path, kind: Details) -> Result<Self, EnglishQuizError> {
        let words: Vec<String> = std::fs::read_to_string(source)
            .map_err(EnglishQuizError::FileError)?
            .lines()
            .map(|line| line.trim().to_string())
            .collect();

        Ok(Self {
            api,
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

    pub fn generate_mcq<const N: usize>(&self, word: &str) -> Result<Mcq<N>, EnglishQuizError> {
        let synonyms_resp = self
            .api
            .get_synonyms(word)
            .map_err(EnglishQuizError::ApiError)?;

        let statement = match self.kind {
            Details::Synonyms => {
                let mut examples_resp = self
                    .api
                    .get_examples(word)
                    .map_err(EnglishQuizError::ApiError)?;

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
                let mut definition_resp = self
                    .api
                    .get_definitions(word)
                    .map_err(EnglishQuizError::ApiError)?;

                if definition_resp.definitions.is_empty()
                    || synonyms_resp.synonyms.len() < N - 1
                    || synonyms_resp.word != definition_resp.word
                    || synonyms_resp.word != word
                {
                    return Err(EnglishQuizError::DataError);
                }

                let definition_index =
                    rand::rng().random_range(0..definition_resp.definitions.len());
                std::mem::take(&mut definition_resp.definitions[definition_index].definition)
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
