use clap::{Parser, ValueEnum};
use std::path::{Path, PathBuf};

use quizgen::{
    english::{EnglishQuiz, EnglishQuizError},
    words_api::{Details, WordsApi},
    QuizMode, QuizType, Section,
};

const WORDS_API_KEY: &str = "WORDS_API_KEY";

fn validate_path(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if path.exists() {
        Ok(path)
    } else {
        Err(format!("File does not exist: {s}"))
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

    #[arg(short, long)]
    length: usize,

    #[arg(short, long, value_parser = validate_path)]
    source: PathBuf,
}

fn quiz(args: QuizArgs) -> anyhow::Result<()> {
    match args.r#type.into() {
        QuizType::English(kind) => {
            let api = WordsApi::new(std::env::var(WORDS_API_KEY)?)?;
            let mut english_quiz = EnglishQuiz::new(api, &args.source, kind)?;
            let mut questions = Vec::with_capacity(args.length);
            while questions.len() <= args.length && english_quiz.available_words() != 0 {
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

            section.save(Path::new("questions.txt"))?;
            report.save(Path::new("report.txt"))?;

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
