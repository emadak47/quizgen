use std::marker;

pub trait Solver<T: Eq + PartialEq> {
    fn solve(&self) -> T;
}

pub struct Answered<T: Eq + PartialEq>(T);
pub struct Unanswered;

pub struct Question<T: Eq + PartialEq, S: Solver<T>, State = Unanswered> {
    style: S,
    state: State,
    _phantom: marker::PhantomData<T>,
}

impl<T: Eq + PartialEq, S: Solver<T>> Question<T, S, Unanswered> {
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
