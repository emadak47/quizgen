use crate::{Quizzer, Solver};

use std::fmt;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Choice {
    A,
    B,
    C,
    D,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, PartialEq, Eq)]
pub struct MCQ {
    sentence: String,
    choices: [String; 4],
    solution: Choice,
}

impl MCQ {
    fn solution(&self) -> &String {
        &self.choices[self.solution as usize]
    }
}

impl Solver<Choice> for MCQ {
    fn solve(&self) -> Choice {
        self.solution
    }
}

impl Quizzer for MCQ {
    fn generate(bank: &[&Self], old: &[&Self]) -> Self
    where
        Self: Sized,
    {
        use rand::prelude::IndexedRandom;

        let available: Vec<&&Self> = bank.iter().filter(|item| !old.contains(item)).collect();

        if let Some(&&chosen) = available.choose(&mut rand::rng()) {
            Self {
                sentence: chosen.sentence.clone(),
                choices: chosen.choices.clone(),
                solution: chosen.solution,
            }
        } else {
            let chosen = bank.choose(&mut rand::rng()).expect("bank cannot be empty");

            Self {
                sentence: chosen.sentence.clone(),
                choices: chosen.choices.clone(),
                solution: chosen.solution,
            }
        }
    }
}

impl fmt::Debug for MCQ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}",
            self.sentence.replacen(self.solution(), "[.....]", 1)
        )?;
        writeln!(f)?;

        for (idx, choice) in self.choices.iter().enumerate() {
            writeln!(
                f,
                "\t{}. {} {}",
                (b'A' + idx as u8) as char,
                if idx == self.solution as usize {
                    "✓"
                } else {
                    " "
                },
                choice
            )?;
        }
        Ok(())
    }
}

impl fmt::Display for MCQ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}",
            self.sentence.replacen(self.solution(), "[.....]", 1)
        )?;
        writeln!(f)?;

        for (idx, choice) in self.choices.iter().enumerate() {
            writeln!(f, "\t{}. {}", (b'A' + idx as u8) as char, choice)?;
        }

        Ok(())
    }
}
