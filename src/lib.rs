pub mod english;
mod mcq;
pub mod words_api;

use clap::ValueEnum;
use serde::Serialize;
use std::{fmt, fs, io, path::Path, str::FromStr, time::Instant};

pub trait Question {
    type Answer: PartialEq + FromStr;

    fn ask(&self) -> impl fmt::Display;
    fn attempt(&self, statement: &str) -> Option<Self::Answer>;
    fn answer(&self) -> Self::Answer;
}

#[derive(Debug)]
pub enum QuizType {
    English(words_api::Details),
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum QuizMode {
    /// Display one question at a time
    #[default]
    Interactive,
    /// Display all questions, then collect anwers
    Batch,
}

pub struct GradeReport<T> {
    start_time: Instant,
    end_time: Instant,
    graded_answers: Vec<(T, Option<T>)>,
}

impl<T: PartialEq> GradeReport<T> {
    fn new(start_time: Instant, end_time: Instant, graded_answers: Vec<(T, Option<T>)>) -> Self {
        Self {
            start_time,
            end_time,
            graded_answers,
        }
    }

    fn calculate_score(&self) -> f64 {
        let total = self.graded_answers.len();
        if total == 0 {
            return 0.0;
        }
        let correct = self
            .graded_answers
            .iter()
            .filter_map(|(a1, a2)| a2.as_ref().map(|ans| ans == a1))
            .filter(|&is_correct| is_correct)
            .count();
        correct as f64 / total as f64 * 100.0
    }

    pub fn save<P>(&self, path: P) -> Result<(), io::Error>
    where
        T: Serialize,
        P: AsRef<Path>,
    {
        let contents = serde_json::to_string_pretty(&self.graded_answers)?;
        fs::write(path, contents)
    }
}

impl<T: PartialEq + fmt::Display> fmt::Display for GradeReport<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Time taken: {:?}", self.end_time - self.start_time)?;
        writeln!(f, "Score: {:.1}%", self.calculate_score())?;
        for (i, (answer, your_answer)) in self.graded_answers.iter().enumerate() {
            match your_answer {
                Some(your_answer) if your_answer == answer => writeln!(f, "{i}. ✔ {answer}")?,
                _ => writeln!(f, "{i}. ✘ {answer}")?,
            }
        }
        Ok(())
    }
}

pub struct Section<Q: Question> {
    questions: Vec<Q>,
}

impl<Q: Question> Section<Q> {
    pub fn new(questions: Vec<Q>) -> Self {
        Self { questions }
    }

    pub fn start_quiz(&self, mode: QuizMode) -> GradeReport<Q::Answer> {
        match mode {
            QuizMode::Interactive => self.interactive_quiz(),
            QuizMode::Batch => self.batch_quiz(),
        }
    }

    fn batch_quiz(&self) -> GradeReport<Q::Answer> {
        let mut answers = Vec::with_capacity(self.questions.len());
        let start_time = Instant::now();

        for (i, question) in self.questions.iter().enumerate() {
            println!("Question {}: {}", i + 1, question.ask());
        }

        for i in 1..=self.questions.len() {
            print!("Enter your answer for question {i}: ");
            io::Write::flush(&mut io::stdout()).unwrap();
            let mut answer = String::new();
            io::stdin().read_line(&mut answer).unwrap();
            match answer.trim().parse::<Q::Answer>() {
                Ok(answer) => answers.push(Some(answer)),
                Err(_) => answers.push(None),
            }
        }

        let end_time = Instant::now();
        let grade_answers = self.grade(answers);

        GradeReport::new(start_time, end_time, grade_answers)
    }

    fn interactive_quiz(&self) -> GradeReport<Q::Answer> {
        let mut answers = Vec::with_capacity(self.questions.len());
        let start_time = Instant::now();

        for (i, question) in self.questions.iter().enumerate() {
            let statement = format!("Question {}: {}", i + 1, question.ask());
            answers.push(question.attempt(&statement));
            println!("\n");
        }

        let end_time = Instant::now();
        let grade_answers = self.grade(answers);

        GradeReport::new(start_time, end_time, grade_answers)
    }

    fn grade(&self, mut answers: Vec<Option<Q::Answer>>) -> Vec<(Q::Answer, Option<Q::Answer>)> {
        answers.resize_with(self.questions.len(), || None);

        self.questions
            .iter()
            .zip(answers)
            .map(|(q, a)| (q.answer(), a))
            .collect()
    }

    pub fn save<P>(&self, path: P) -> Result<(), io::Error>
    where
        Q: Serialize,
        P: AsRef<Path>,
    {
        let contents = serde_json::to_string_pretty(&self.questions)?;
        fs::write(path, contents)
    }
}
