pub mod mcq;
pub mod words_api;

use std::{
    error::Error,
    fmt::{self, Debug, Display},
    io::{self, Write},
    marker,
    str::FromStr,
};

pub enum QuizMode {
    Interactive, // Display one question at a time
    Batch,       // Display all questions, then collect anwers
}

impl FromStr for QuizMode {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "interative" | "Interactive" | "1" => Ok(Self::Interactive),
            "batch" | "Batch" | "2" => Ok(Self::Batch),
            _ => Err(format!("Cannot parse {s}").into()),
        }
    }
}

fn interactive_quiz<T, S>(section: Section<T, S>) -> f64
where
    T: FromStr + Eq + PartialEq + Debug,
    <T as FromStr>::Err: Display,
    S: Solver<T> + Quizzer + Debug + Display,
{
    let mut answers = Vec::new();

    for (i, question) in section.questions.iter().enumerate() {
        println!("Question {} of {}", i + 1, section.questions.len());
        println!("{question}");

        print!("Enter your answer: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim().parse::<T>() {
            Ok(answer) => answers.push(answer),
            Err(e) => {
                println!("Invalid input: {e}. Skipping this question.");
                // We'll skip invalid answers by not adding them to the vector
                // The answer method will handle missing answers with None
            }
        }
    }

    section.answer(answers).grade()
}

fn batch_quiz<T, S>(section: Section<T, S>) -> f64
where
    T: FromStr + Eq + PartialEq + Debug,
    <T as FromStr>::Err: Display,
    S: Solver<T> + Quizzer + Debug + Display,
{
    println!("{section}"); // display the questions

    let mut answers = Vec::new();
    for i in 1..=section.questions.len() {
        print!("Enter your answer for question {i}: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim().parse::<T>() {
            Ok(answer) => answers.push(answer),
            Err(e) => {
                println!("Invalid input: {e}. Skipping this question.");
                // We'll skip invalid answers by not adding them to the vector
                // The answer method will handle missing answers with None
            }
        }
    }

    section.answer(answers).grade()
}

pub fn quiz<T, S>(n: usize, mut section: Section<T, S>, quiz_mode: QuizMode) -> f64
where
    T: FromStr + Eq + PartialEq + Debug,
    <T as FromStr>::Err: Display,
    S: Solver<T> + Quizzer + Debug + Display,
{
    section.prepare(n);

    match quiz_mode {
        QuizMode::Batch => batch_quiz(section),
        QuizMode::Interactive => interactive_quiz(section),
    }
}

pub trait Quizzer {
    fn generate(bank: &[&Self], old: &[&Self]) -> Self
    where
        Self: Sized;
}

pub trait Solver<T: Eq + PartialEq> {
    fn solve(&self) -> T;
}

pub struct Answered<T: Eq + PartialEq>(Option<T>);
pub struct Unanswered;

pub struct Question<T: Eq + PartialEq, S: Solver<T> + Quizzer, State = Unanswered> {
    style: S,
    state: State,
    _phantom: marker::PhantomData<T>,
}

impl<T: Eq + PartialEq, S: Solver<T> + Quizzer> Question<T, S, Unanswered> {
    pub fn new(style: S) -> Self {
        Self {
            style,
            state: Unanswered,
            _phantom: marker::PhantomData::<T>,
        }
    }

    pub fn answer(self, answer: Option<T>) -> Question<T, S, Answered<T>> {
        Question {
            style: self.style,
            state: Answered(answer),
            _phantom: marker::PhantomData::<T>,
        }
    }
}

impl<T: Eq + PartialEq, S: Solver<T> + Quizzer> Question<T, S, Answered<T>> {
    fn mark(&self) -> bool {
        self.state
            .0
            .as_ref()
            .map(|a| &self.style.solve() == a)
            .unwrap_or(false)
    }
}

impl<T: Eq + PartialEq, S: Solver<T> + Quizzer + Debug> Debug for Question<T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:#?}", self.style)
    }
}

impl<T: Eq + PartialEq, S: Solver<T> + Quizzer + Display> Display for Question<T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.style)
    }
}

#[derive(Default)]
pub struct Section<T: Eq + PartialEq, S: Solver<T> + Quizzer, State = Unanswered> {
    bank: Vec<Question<T, S, Answered<T>>>,
    questions: Vec<Question<T, S, State>>,
}

impl<T: Eq + PartialEq, S: Solver<T> + Quizzer> Section<T, S, Unanswered> {
    pub fn new(bank: Vec<Question<T, S, Answered<T>>>) -> Self {
        Self {
            bank,
            questions: vec![],
        }
    }

    fn ask(&self) -> Question<T, S> {
        let bank: Vec<&S> = self.bank.iter().map(|q| &q.style).collect();
        let old: Vec<&S> = self.questions.iter().map(|q| &q.style).collect();
        Question::new(S::generate(&bank, &old))
    }

    fn prepare(&mut self, n: usize) {
        for _ in 1..=n {
            self.questions.push(self.ask())
        }
    }

    fn answer(self, answers: Vec<T>) -> Section<T, S, Answered<T>> {
        let mut answers: Vec<_> = answers.into_iter().map(Option::Some).collect();
        answers.resize_with(self.questions.len(), || None);

        Section {
            bank: self.bank,
            questions: self
                .questions
                .into_iter()
                .zip(answers)
                .map(|(q, a)| q.answer(a))
                .collect(),
        }
    }
}

impl<T: Eq + PartialEq, S: Solver<T> + Quizzer> Section<T, S, Answered<T>> {
    fn grade(&self) -> f64 {
        (self
            .questions
            .iter()
            .map(|q| q.mark())
            .filter(|m| *m)
            .count() as f64
            / self.questions.len() as f64)
            * 100.0
    }
}

impl<T: Eq + PartialEq, S: Solver<T> + Quizzer + Debug> Debug for Section<T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Total Questions: {}\n", self.questions.len())?;
        for (i, q) in self.questions.iter().enumerate() {
            writeln!(f, "Question {}", i + 1)?;
            writeln!(f, "{q:?}")?;
        }
        Ok(())
    }
}

impl<T: Eq + PartialEq, S: Solver<T> + Quizzer + Display> Display for Section<T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Answer the following questions:\n")?;
        for (i, q) in self.questions.iter().enumerate() {
            writeln!(f, "Question {}", i + 1)?;
            write!(f, "{q}")?;
        }
        Ok(())
    }
}
