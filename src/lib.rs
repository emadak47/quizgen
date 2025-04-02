pub struct Answered<T: Eq + PartialEq>(T);
pub struct Unanswered;

pub struct Question<State = Unanswered> {
    state: State,
}

impl Question<Unanswered> {
    fn new() -> Self {
        Self { state: Unanswered }
    }

    fn answer<T: Eq + PartialEq>(self, answer: T) -> Question<Answered<T>> {
        Question {
            state: Answered(answer),
        }
    }
}
