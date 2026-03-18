use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum Choice {
    A,
    B,
    C,
    D,
}

impl FromStr for Choice {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "A" | "a" => Ok(Choice::A),
            "B" | "b" => Ok(Choice::B),
            "C" | "c" => Ok(Choice::C),
            "D" | "d" => Ok(Choice::D),
            _ => Err(format!("Invalid choice: '{s}'").into()),
        }
    }
}

impl TryFrom<usize> for Choice {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Choice::A),
            1 => Ok(Choice::B),
            2 => Ok(Choice::C),
            3 => Ok(Choice::D),
            _ => Err(format!("Invalid choice: '{value}'").into()),
        }
    }
}

impl fmt::Display for Choice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Choice::A => "A",
            Choice::B => "B",
            Choice::C => "C",
            Choice::D => "D",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mcq<const N: usize> {
    statement: String,
    #[serde(bound(serialize = "[String; N]: Serialize"))]
    #[serde(bound(deserialize = "[String; N]: Deserialize<'de>"))]
    choices: [String; N],
    solution: Choice,
}

impl<const N: usize> Mcq<N> {
    pub fn new(statement: String, choices: [String; N], solution: Choice) -> Self {
        Self {
            statement,
            choices,
            solution,
        }
    }

    pub fn statement(&self) -> &str {
        &self.statement
    }

    pub fn choices(&self) -> &[String; N] {
        &self.choices
    }

    pub fn solution(&self) -> Choice {
        self.solution
    }
}
