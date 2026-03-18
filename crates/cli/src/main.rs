use clap::{Parser, ValueEnum};
use inquire::Select;
use rand::seq::SliceRandom;
use std::{
    fs::{self, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
    str::FromStr,
    time::Instant,
};

use quizgen_core::{
    english::{Details, EnglishQuiz},
    mcq::{Choice, Mcq},
    webster::WebsterApi,
    words_api::WordsApi,
    GradeReport, QuizgenError,
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
enum QuizType {
    Synonyms,
    Antonyms,
    Definitions,
    Examples,
}

impl From<QuizType> for Details {
    fn from(t: QuizType) -> Self {
        match t {
            QuizType::Synonyms => Details::Synonyms,
            QuizType::Antonyms => Details::Antonyms,
            QuizType::Definitions => Details::Definitions,
            QuizType::Examples => Details::Examples,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum QuizMode {
    #[default]
    Interactive,
    Batch,
}

#[derive(Debug, Parser)]
#[command(version, about = "A CLI to construct a quiz")]
struct QuizArgs {
    #[arg(long, value_enum)]
    r#type: QuizType,

    #[arg(long, value_enum)]
    mode: QuizMode,

    #[arg(short, long, value_parser = validate_length)]
    length: usize,

    #[arg(short, long, value_parser = validate_path, env = "SOURCE_DIR")]
    source: PathBuf,

    #[arg(short, long, default_value_t = false)]
    prev: bool,
}

fn load_questions() -> Result<Vec<Mcq<4>>, io::Error> {
    let reader = BufReader::new(File::open(Path::new(QUESTIONS_FILE))?);
    let questions: Vec<Mcq<4>> = serde_json::from_reader(reader)?;

    let reader = BufReader::new(File::open(Path::new(ANSWERS_FILE))?);
    let answers: Vec<(Choice, Option<Choice>)> = serde_json::from_reader(reader)?;

    Ok(questions
        .into_iter()
        .zip(answers)
        .filter_map(|(q, (a1, a2))| a2.as_ref().filter(|&a2| a2 != &a1).map(|_| q))
        .collect())
}

async fn generate_questions(
    quiz: &mut EnglishQuiz,
    count: usize,
    prev: Option<Vec<Mcq<4>>>,
) -> Result<Vec<Mcq<4>>, QuizgenError> {
    let mut questions = prev.unwrap_or_default();
    questions.reserve(count);
    while questions.len() < count {
        match quiz.gen_rand_mcq::<4>().await {
            Some(Ok(q)) => questions.push(q),
            Some(Err(QuizgenError::DataError)) => continue,
            Some(Err(e)) => return Err(e),
            None => break,
        }
    }
    Ok(questions)
}

fn interactive_quiz(questions: &[Mcq<4>]) -> GradeReport<Choice> {
    let start_time = Instant::now();
    let mut answers = Vec::with_capacity(questions.len());

    for (i, question) in questions.iter().enumerate() {
        let solution = &question.choices()[question.solution() as usize];
        let statement = question.statement().replacen(solution, "[.....]", 1);
        let prompt = format!("Question {}: {}", i + 1, statement);

        let options: Vec<String> = question
            .choices()
            .iter()
            .enumerate()
            .map(|(idx, ch)| format!("\t{}. {}", (b'A' + idx as u8) as char, ch))
            .collect();

        let answer = Select::new(&prompt, options)
            .prompt()
            .ok()
            .and_then(|s| s.get(0..2).and_then(|ch| Choice::from_str(ch).ok()));
        answers.push(answer);
        println!("\n");
    }

    let end_time = Instant::now();
    let graded = grade(questions, answers);
    GradeReport::new(start_time, end_time, graded)
}

fn batch_quiz(questions: &[Mcq<4>]) -> GradeReport<Choice> {
    let start_time = Instant::now();
    let mut answers = Vec::with_capacity(questions.len());

    for (i, question) in questions.iter().enumerate() {
        let solution = &question.choices()[question.solution() as usize];
        let statement = question.statement().replacen(solution, "[.....]", 1);
        println!("Question {}: {}", i + 1, statement);
    }

    for i in 1..=questions.len() {
        print!("Enter your answer for question {i}: ");
        io::Write::flush(&mut io::stdout()).unwrap();
        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();
        match line.trim().parse::<Choice>() {
            Ok(answer) => answers.push(Some(answer)),
            Err(_) => answers.push(None),
        }
    }

    let end_time = Instant::now();
    let graded = grade(questions, answers);
    GradeReport::new(start_time, end_time, graded)
}

fn grade(questions: &[Mcq<4>], mut answers: Vec<Option<Choice>>) -> Vec<(Choice, Option<Choice>)> {
    answers.resize_with(questions.len(), || None);
    questions
        .iter()
        .zip(answers)
        .map(|(q, a)| (q.solution(), a))
        .collect()
}

async fn quiz(args: QuizArgs) -> anyhow::Result<()> {
    let kind: Details = args.r#type.into();

    let prev_questions: Option<Vec<Mcq<4>>> = if args.prev {
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

    let questions = generate_questions(&mut english_quiz, args.length, prev_questions).await?;

    let report = match args.mode {
        QuizMode::Interactive => interactive_quiz(&questions),
        QuizMode::Batch => batch_quiz(&questions),
    };

    println!("\n\n{report}");

    let questions_json = serde_json::to_string_pretty(&questions)?;
    fs::write(Path::new(QUESTIONS_FILE), questions_json)?;

    let answers_json = serde_json::to_string_pretty(&report.graded_answers)?;
    fs::write(Path::new(ANSWERS_FILE), answers_json)?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    quiz(QuizArgs::parse()).await
}
