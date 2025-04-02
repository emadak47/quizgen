pub mod mcq;

use std::{fmt, marker};

pub fn quiz<T: Eq + PartialEq + fmt::Debug, S: Solver<T> + Quizzer + fmt::Debug + fmt::Display>(
    n: usize,
    mut section: Section<T, S>,
) -> f64 {
    section.prepare(n);
    println!("{section}"); // display the questions
    let answers: Vec<T> = Vec::new(); // accept answers from user
    section.answer(answers).grade()
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

impl<T: Eq + PartialEq, S: Solver<T> + Quizzer + fmt::Debug> fmt::Debug for Question<T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:#?}", self.style)
    }
}

impl<T: Eq + PartialEq, S: Solver<T> + Quizzer + fmt::Display> fmt::Display for Question<T, S> {
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

impl<T: Eq + PartialEq, S: Solver<T> + Quizzer + fmt::Debug> fmt::Debug for Section<T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Total Questions: {}\n", self.questions.len())?;
        for (i, q) in self.questions.iter().enumerate() {
            writeln!(f, "Question {}", i + 1)?;
            writeln!(f, "{:?}", q)?;
        }
        Ok(())
    }
}

impl<T: Eq + PartialEq, S: Solver<T> + Quizzer + fmt::Display> fmt::Display for Section<T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Answer the following questions:\n")?;
        for (i, q) in self.questions.iter().enumerate() {
            writeln!(f, "Question {}", i + 1)?;
            write!(f, "{}", q)?;
        }
        Ok(())
    }
}
