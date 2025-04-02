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
