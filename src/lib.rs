use std::marker;

pub trait Quizzer {
    fn generate(old: &[&Self]) -> Self
    where
        Self: Sized;
}

pub trait Solver<T: Eq + PartialEq> {
    fn solve(&self) -> T;
}

pub struct Answered<T: Eq + PartialEq>(T);
pub struct Unanswered;

pub struct Question<T: Eq + PartialEq, S: Solver<T> + Quizzer, State = Unanswered> {
    style: S,
    state: State,
    _phantom: marker::PhantomData<T>,
}

impl<T: Eq + PartialEq, S: Solver<T> + Quizzer> Question<T, S, Unanswered> {
    fn new(style: S) -> Self {
        Self {
            style,
            state: Unanswered,
            _phantom: marker::PhantomData::<T>,
        }
    }

    fn answer(self, answer: T) -> Question<T, S, Answered<T>> {
        Question {
            style: self.style,
            state: Answered(answer),
            _phantom: marker::PhantomData::<T>,
        }
    }
}

impl<T: Eq + PartialEq, S: Solver<T> + Quizzer> Question<T, S, Answered<T>> {
    fn mark(&self) -> bool {
        self.state.0 == self.style.solve()
    }
}

#[derive(Default)]
pub struct Section<T: Eq + PartialEq, S: Solver<T> + Quizzer, State = Unanswered> {
    questions: Vec<Question<T, S, State>>,
}

impl<T: Eq + PartialEq, S: Solver<T> + Quizzer> Section<T, S, Unanswered> {
    pub fn new() -> Self {
        Self { questions: vec![] }
    }

    fn ask(&self) -> Question<T, S> {
        let old: Vec<&S> = self.questions.iter().map(|q| &q.style).collect();
        Question::new(S::generate(&old))
    }

    fn prepare(&mut self, n: usize) {
        for _ in 1..=n {
            self.questions.push(self.ask())
        }
    }

    fn answer(self, answers: Vec<T>) -> Section<T, S, Answered<T>> {
        Section {
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
