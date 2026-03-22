pub mod english;
pub mod mcq;
pub mod webster;
pub mod words_api;

use std::time::Duration;

use crate::mcq::{Choice, Mcq};

#[derive(thiserror::Error, Debug)]
pub enum QuizgenError {
    #[error("API error")]
    ApiError(anyhow::Error),
    #[error("Data is invalid")]
    DataError,
    #[error("File error: {0}")]
    FileError(#[from] std::io::Error),
}

pub struct GradedQuiz<'a, const N: usize> {
    questions: &'a [Mcq<N>],
    pub answers: &'a [Option<Choice>],
    pub elapsed: Duration,
}

pub struct QuestionGrade<'a> {
    pub correct: bool,
    pub correct_answer: &'a str,
    pub your_answer: Option<&'a str>,
}

impl<'a, const N: usize> GradedQuiz<'a, N> {
    pub fn new(questions: &'a [Mcq<N>], answers: &'a [Option<Choice>], elapsed: Duration) -> Self {
        Self {
            questions,
            answers,
            elapsed,
        }
    }

    pub fn score(&self) -> f64 {
        let total = self.questions.len();
        if total == 0 {
            return 0.0;
        }
        let correct = self.iter().filter(|g| g.correct).count();
        correct as f64 / total as f64 * 100.0
    }

    pub fn iter(&self) -> impl Iterator<Item = QuestionGrade<'_>> + '_ {
        self.questions.iter().zip(self.answers).map(|(q, a)| {
            let correct_choice = q.solution();
            let is_correct = a.is_some_and(|a| a == correct_choice);
            let correct_answer = q.choices()[correct_choice as usize].as_str();
            let your_answer = a.map(|a| q.choices()[a as usize].as_str());
            QuestionGrade {
                correct: is_correct,
                correct_answer,
                your_answer,
            }
        })
    }
}
