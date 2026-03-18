use async_trait::async_trait;
use rand::prelude::*;
use serde::Deserialize;
use std::{collections::HashSet, path::Path};

use crate::{
    mcq::{Choice, Mcq},
    QuizgenError,
};

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

#[async_trait]
pub trait EnglishApi: Send + Sync {
    async fn get_definitions(&self, word: &str) -> anyhow::Result<DefinitionResponse>;
    async fn get_examples(&self, word: &str) -> anyhow::Result<ExampleResponse>;
    async fn get_synonyms(&self, word: &str) -> anyhow::Result<SynonymResponse>;
    async fn get_antonyms(&self, word: &str) -> anyhow::Result<AntonymResponse>;
}

fn select_random<T, const N: usize>(buf: &mut Vec<T>, rng: &mut ThreadRng) -> Option<[T; N]> {
    if buf.len() < N {
        return None;
    }

    Some(core::array::from_fn(|_| {
        let rnd_idx = rng.random_range(..buf.len());
        buf.swap_remove(rnd_idx)
    }))
}

fn cap_first_char(word: &mut str) {
    word.make_ascii_lowercase();
    if let Some(first) = word.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
}

pub struct EnglishQuiz {
    apis: [Box<dyn EnglishApi>; 2],
    kind: Details,
    words: Vec<String>,
}

impl EnglishQuiz {
    pub fn new(
        apis: [Box<dyn EnglishApi>; 2],
        source: &Path,
        kind: Details,
    ) -> Result<Self, QuizgenError> {
        let mut words = HashSet::new();
        let dir = std::fs::read_dir(source).map_err(QuizgenError::FileError)?;

        for entry in dir {
            let path = entry.map_err(QuizgenError::FileError)?.path();

            if !path.is_file() {
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("txt") {
                continue;
            }

            let contents = if let Ok(c) = std::fs::read_to_string(&path) {
                c
            } else {
                continue;
            };

            for line in contents.lines() {
                let word = line.trim().to_string();
                if !word.is_empty() {
                    words.insert(word);
                }
            }
        }

        Ok(Self {
            apis,
            kind,
            words: words.into_iter().collect(),
        })
    }

    async fn try_get_definitions(&self, word: &str) -> Result<DefinitionResponse, QuizgenError> {
        let mut last_err = None;
        for api in &self.apis {
            match api.get_definitions(word).await {
                Ok(t) => return Ok(t),
                Err(e) => last_err = Some(e),
            }
        }
        Err(QuizgenError::ApiError(last_err.unwrap()))
    }

    async fn try_get_examples(&self, word: &str) -> Result<ExampleResponse, QuizgenError> {
        let mut last_err = None;
        for api in &self.apis {
            match api.get_examples(word).await {
                Ok(t) => return Ok(t),
                Err(e) => last_err = Some(e),
            }
        }
        Err(QuizgenError::ApiError(last_err.unwrap()))
    }

    async fn try_get_synonyms(&self, word: &str) -> Result<SynonymResponse, QuizgenError> {
        let mut last_err = None;
        for api in &self.apis {
            match api.get_synonyms(word).await {
                Ok(t) => return Ok(t),
                Err(e) => last_err = Some(e),
            }
        }
        Err(QuizgenError::ApiError(last_err.unwrap()))
    }

    async fn try_get_antonyms(&self, word: &str) -> Result<AntonymResponse, QuizgenError> {
        let mut last_err = None;
        for api in &self.apis {
            match api.get_antonyms(word).await {
                Ok(t) => return Ok(t),
                Err(e) => last_err = Some(e),
            }
        }
        Err(QuizgenError::ApiError(last_err.unwrap()))
    }

    pub async fn gen_rand_mcq<const N: usize>(&mut self) -> Option<Result<Mcq<N>, QuizgenError>> {
        let word_opt = select_random::<_, 1>(&mut self.words, &mut rand::rng());
        if let Some([word]) = word_opt {
            match self.gen_mcq(&word).await {
                Ok(q) => return Some(Ok(q)),
                Err(e @ (QuizgenError::DataError | QuizgenError::ApiError(_))) => {
                    return Some(Err(e))
                }
                _ => unreachable!(),
            }
        }
        None
    }

    async fn gen_mcq<const N: usize>(&mut self, word: &str) -> Result<Mcq<N>, QuizgenError> {
        let (word, statement) = match self.kind {
            Details::Synonyms => {
                let SynonymResponse { word, mut synonyms } = self.try_get_synonyms(word).await?;

                let synonyms: [_; N] = select_random(&mut synonyms, &mut rand::rng())
                    .ok_or(QuizgenError::DataError)?;
                let statement = synonyms.join(", ");

                (word, statement)
            }
            Details::Antonyms => {
                let AntonymResponse { word, mut antonyms } = self.try_get_antonyms(word).await?;

                let antonyms: [_; N] = select_random(&mut antonyms, &mut rand::rng())
                    .ok_or(QuizgenError::DataError)?;
                let statement = antonyms.join(", ");

                (word, statement)
            }
            Details::Examples => {
                let ExampleResponse { word, mut examples } = self.try_get_examples(word).await?;

                let [statement] = select_random(&mut examples, &mut rand::rng())
                    .ok_or_else(|| QuizgenError::DataError)?;

                (word, statement)
            }
            Details::Definitions => {
                let DefinitionResponse {
                    word,
                    mut definitions,
                } = self.try_get_definitions(word).await?;

                let [statement] = select_random(&mut definitions, &mut rand::rng())
                    .ok_or_else(|| QuizgenError::DataError)?;

                (word, statement)
            }
        };

        let mut choices: [_; N] =
            select_random(&mut self.words, &mut rand::rng()).ok_or(QuizgenError::DataError)?;
        let rnd_idx = rand::rng().random_range(..N);
        let solution = Choice::try_from(rnd_idx).expect("Choice is valid");

        choices[rnd_idx] = word;
        choices.iter_mut().for_each(|ch| cap_first_char(ch));

        Ok(Mcq::new(statement, choices, solution))
    }
}
