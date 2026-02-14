use rand::prelude::*;
use serde::Deserialize;
use std::path::Path;

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

pub trait EnglishApi {
    fn get_definitions(&self, word: &str) -> anyhow::Result<DefinitionResponse>;
    fn get_examples(&self, word: &str) -> anyhow::Result<ExampleResponse>;
    fn get_synonyms(&self, word: &str) -> anyhow::Result<SynonymResponse>;
    fn get_antonyms(&self, word: &str) -> anyhow::Result<AntonymResponse>;
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
        let words: Vec<String> = std::fs::read_to_string(source)
            .map_err(QuizgenError::FileError)?
            .lines()
            .map(|line| line.trim().to_string())
            .collect();

        Ok(Self { apis, kind, words })
    }

    fn try_get<F, T>(&self, f: F) -> Result<T, QuizgenError>
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
        Err(QuizgenError::ApiError(last_err.unwrap()))
    }

    pub fn gen_rand_mcq<const N: usize>(&mut self) -> Option<Result<Mcq<N>, QuizgenError>> {
        if let Some([word]) = select_random(&mut self.words, &mut rand::rng()) {
            match self.gen_mcq(&word) {
                Ok(q) => return Some(Ok(q)),
                Err(e @ (QuizgenError::DataError | QuizgenError::ApiError(_))) => {
                    return Some(Err(e))
                }
                _ => unreachable!(),
            }
        }
        None
    }

    fn gen_mcq<const N: usize>(&mut self, word: &str) -> Result<Mcq<N>, QuizgenError> {
        let mut rng = rand::rng();

        let (word, statement) = match self.kind {
            Details::Synonyms => {
                let SynonymResponse { word, mut synonyms } =
                    self.try_get(|api| api.get_synonyms(word))?;

                let synonyms: [_; N] =
                    select_random(&mut synonyms, &mut rng).ok_or(QuizgenError::DataError)?;
                let statement = synonyms.join(", ");

                (word, statement)
            }
            Details::Antonyms => {
                let AntonymResponse { word, mut antonyms } =
                    self.try_get(|api| api.get_antonyms(word))?;

                let antonyms: [_; N] =
                    select_random(&mut antonyms, &mut rng).ok_or(QuizgenError::DataError)?;
                let statement = antonyms.join(", ");

                (word, statement)
            }
            Details::Examples => {
                let ExampleResponse { word, mut examples } =
                    self.try_get(|api| api.get_examples(word))?;

                let [statement] = select_random(&mut examples, &mut rng)
                    .ok_or_else(|| QuizgenError::DataError)?;

                (word, statement)
            }
            Details::Definitions => {
                let DefinitionResponse {
                    word,
                    mut definitions,
                } = self.try_get(|api| api.get_definitions(word))?;

                let [statement] = select_random(&mut definitions, &mut rng)
                    .ok_or_else(|| QuizgenError::DataError)?;

                (word, statement)
            }
        };

        let mut choices: [_; N] =
            select_random(&mut self.words, &mut rng).ok_or(QuizgenError::DataError)?;
        let rnd_idx = rng.random_range(..N);
        let solution = Choice::try_from(rnd_idx).expect("Choice is valid");
        choices[rnd_idx] = word;

        Ok(Mcq::new(statement, choices, solution))
    }
}
