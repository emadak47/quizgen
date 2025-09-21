use crate::{Quizzer, Solver};

use std::{
    fmt::{self, Debug, Display},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Choice {
    A,
    B,
    C,
    D,
}

impl FromStr for Choice {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "A" => Ok(Choice::A),
            "B" => Ok(Choice::B),
            "C" => Ok(Choice::C),
            "D" => Ok(Choice::D),
            _ => Err(format!(
                "Invalid choice: '{}'. Please enter A, B, C, or D.",
                s.trim()
            )),
        }
    }
}

impl TryFrom<usize> for Choice {
    type Error = String;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Choice::A),
            1 => Ok(Choice::B),
            2 => Ok(Choice::C),
            3 => Ok(Choice::D),
            _ => Err(format!("Invalid choice: '{}'", value)),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, PartialEq, Eq)]
pub struct MCQ {
    sentence: String,
    choices: [String; 4],
    solution: Choice,
}

impl MCQ {
    pub fn new(
        sentence: impl Into<String>,
        choices: [impl Into<String>; 4],
        solution: Choice,
    ) -> Self {
        Self {
            sentence: sentence.into(),
            choices: choices.map(Into::into),
            solution,
        }
    }

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

impl Debug for MCQ {
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

impl Display for MCQ {
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
