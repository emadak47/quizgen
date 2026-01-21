use clap::{Parser, ValueEnum};
use rand::seq::SliceRandom;
use serde::Deserialize;
use std::{
    fs::File,
    io::{self, BufReader},
    path::{Path, PathBuf},
};

use quizgen::{
    english::{EnglishQuiz, EnglishQuizError},
    words_api::{Details, WordsApi},
    Question, QuizMode, QuizType, Section,
};

const WORDS_API_KEY: &str = "WORDS_API_KEY";
const ANSWERS_FILE: &str = "answers.txt";
const QUESTIONS_FILE: &str = "questions.txt";

fn validate_path(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if path.exists() {
        Ok(path)
    } else {
        Err(format!("File does not exist: {s}"))
    }
}

fn validate_length(s: &str) -> Result<usize, String> {
    let length: usize = s.parse().map_err(|_| "Not a valid number".to_string())?;
    if length > 0 {
        Ok(length)
    } else {
        Err("Length must be greater than 0".to_string())
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum QuizTypeCli {
    /// English quiz with synonyms
    EnglishSynonyms,
    /// English quiz with definitions
    EnglishDefinitions,
}

impl From<QuizTypeCli> for QuizType {
    fn from(cli_type: QuizTypeCli) -> Self {
        match cli_type {
            QuizTypeCli::EnglishSynonyms => QuizType::English(Details::Synonyms),
            QuizTypeCli::EnglishDefinitions => QuizType::English(Details::Definitions),
        }
    }
}

#[derive(Debug, Parser)]
#[command(version, about = "A CLI to construct a quiz")]
struct QuizArgs {
    #[arg(long, value_enum)]
    r#type: QuizTypeCli,

    #[arg(long, value_enum)]
    mode: QuizMode,

    #[arg(short, long, value_parser = validate_length)]
    length: usize,

    #[arg(short, long, value_parser = validate_path)]
    source: PathBuf,
}

/// Loads questions from previous quiz which were unanswered or
/// whose answers were incorrect
fn load_questions<Q>() -> Result<Vec<Q>, io::Error>
where
    Q: Question + for<'a> Deserialize<'a>,
    Q::Answer: for<'a> Deserialize<'a>,
{
    let path = Path::new(QUESTIONS_FILE);
    let reader = BufReader::new(File::open(path)?);
    let questions: Vec<Q> = serde_json::from_reader(reader)?;

    let path = Path::new(ANSWERS_FILE);
    let reader = BufReader::new(File::open(path)?);
    let answers: Vec<(Q::Answer, Option<Q::Answer>)> = serde_json::from_reader(reader)?;

    Ok(questions
        .into_iter()
        .zip(answers)
        .filter_map(|(q, (a1, a2))| a2.as_ref().filter(|&a2| a2 != &a1).map(|_| q))
        .collect())
}

fn quiz(args: QuizArgs) -> anyhow::Result<()> {
    match args.r#type.into() {
        QuizType::English(kind) => {
            let api = WordsApi::new(std::env::var(WORDS_API_KEY)?)?;

            let mut questions = match load_questions() {
                Ok(mut questions) => {
                    questions.shuffle(&mut rand::rng());
                    questions.truncate(args.length / 5);
                    questions.reserve(args.length);
                    questions
                }
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::NotFound | io::ErrorKind::UnexpectedEof
                    ) =>
                {
                    Vec::with_capacity(args.length)
                }
                Err(e) => return Err(e.into()),
            };

            let mut english_quiz = EnglishQuiz::new(api, &args.source, kind)?;

            while questions.len() < args.length && english_quiz.available_words() != 0 {
                let word = match english_quiz.select_word() {
                    Ok(word) => word.to_lowercase(),
                    Err(e) => match e {
                        EnglishQuizError::ApiError(e) => return Err(e),
                        EnglishQuizError::DataError => continue,
                        EnglishQuizError::FileError(e) => return Err(e.into()),
                    },
                };

                let question = match english_quiz.generate_mcq::<4>(&word) {
                    Ok(question) => question,
                    Err(e) => match e {
                        EnglishQuizError::ApiError(e) => return Err(e),
                        EnglishQuizError::DataError => continue,
                        EnglishQuizError::FileError(e) => return Err(e.into()),
                    },
                };
                questions.push(question);
            }

            let section = Section::new(questions);
            let report = section.start_quiz(args.mode);
            println!("\n\n{report}");

            section.save(Path::new(QUESTIONS_FILE))?;
            report.save(Path::new(ANSWERS_FILE))?;

            Ok(())
        }
    }
}

fn main() -> anyhow::Result<()> {
    quiz(QuizArgs::parse())
}

/*
Answer the following questions:

Question 1
Hello [.....]! Welcome to quizgen.

        A. World
        B. Universe
        C. Galaxy
        D. Planet
 */
