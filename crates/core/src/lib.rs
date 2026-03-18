pub mod english;
pub mod mcq;
pub mod webster;
pub mod words_api;

use std::{fmt, time::Instant};

#[derive(thiserror::Error, Debug)]
pub enum QuizgenError {
    #[error("API error")]
    ApiError(anyhow::Error),
    #[error("Data is invalid")]
    DataError,
    #[error("File error: {0}")]
    FileError(#[from] std::io::Error),
}

pub struct GradeReport<T> {
    pub start_time: Instant,
    pub end_time: Instant,
    pub graded_answers: Vec<(T, Option<T>)>,
}

impl<T: PartialEq> GradeReport<T> {
    pub fn new(
        start_time: Instant,
        end_time: Instant,
        graded_answers: Vec<(T, Option<T>)>,
    ) -> Self {
        Self {
            start_time,
            end_time,
            graded_answers,
        }
    }

    pub fn calculate_score(&self) -> f64 {
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
}

impl<T: PartialEq + fmt::Display> fmt::Display for GradeReport<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Time taken: {:?}", self.end_time - self.start_time)?;
        writeln!(f, "Score: {:.1}%", self.calculate_score())?;
        for (i, (answer, your_answer)) in self.graded_answers.iter().enumerate() {
            match your_answer {
                Some(your_answer) if your_answer == answer => writeln!(f, "{}. ✔ {answer}", i + 1)?,
                _ => writeln!(f, "{}. ✘ {answer}", i + 1)?,
            }
        }
        Ok(())
    }
}
