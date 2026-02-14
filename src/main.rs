use clap::{Parser, ValueEnum};
use rand::seq::SliceRandom;
use serde::Deserialize;
use std::{
    fs::File,
    io::{self, BufReader},
    path::{Path, PathBuf},
};

use quizgen::{
    english::{Details, EnglishQuiz},
    webster::WebsterApi,
    words_api::WordsApi,
    Question, QuizMode, QuizType, Section,
};

const WORDS_API_KEY: &str = "WORDS_API_KEY";
const COLLEGIATE_API_KEY: &str = "COLLEGIATE_API_KEY";
const THESAURUS_API_KEY: &str = "THESAURUS_API_KEY";

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
    Synonyms,
    Antonyms,
    Definitions,
    Examples,
}

impl From<QuizTypeCli> for QuizType {
    fn from(cli_type: QuizTypeCli) -> Self {
        match cli_type {
            QuizTypeCli::Synonyms => QuizType::English(Details::Synonyms),
            QuizTypeCli::Antonyms => QuizType::English(Details::Antonyms),
            QuizTypeCli::Definitions => QuizType::English(Details::Definitions),
            QuizTypeCli::Examples => QuizType::English(Details::Examples),
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

    #[arg(short, long, default_value_t = false)]
    prev: bool,
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
            let prev_questions: Option<Vec<_>> = if args.prev {
                match load_questions() {
                    Ok(mut questions) => {
                        questions.shuffle(&mut rand::rng());
                        questions.truncate(args.length / 5);
                        Some(questions)
                    }
                    Err(e)
                        if matches!(
                            e.kind(),
                            io::ErrorKind::NotFound | io::ErrorKind::UnexpectedEof
                        ) =>
                    {
                        None
                    }
                    Err(e) => return Err(e.into()),
                }
            } else {
                None
            };

            let words_api = WordsApi::new(std::env::var(WORDS_API_KEY)?)?;
            let webster_api = WebsterApi::new(
                std::env::var(COLLEGIATE_API_KEY)?,
                std::env::var(THESAURUS_API_KEY)?,
            )?;
            let mut english_quiz = EnglishQuiz::new(
                [Box::new(words_api), Box::new(webster_api)],
                &args.source,
                kind,
            )?;

            let section = Section::from_quizzer(args.length, prev_questions, || {
                english_quiz.gen_rand_mcq::<4>()
            })?;
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
