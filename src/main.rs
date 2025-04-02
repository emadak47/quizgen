use quizgen::{mcq, Question, Section};

fn main() {
    let answer = mcq::Choice::A;
    let mcq = mcq::MCQ::new(
        "Hello World! Welcome to quizgen.",
        ["World", "Universe", "Galaxy", "Planet"],
        answer,
    );

    let question = Question::new(mcq).answer(Some(answer));
    let questions = vec![question];

    let section: Section<mcq::Choice, mcq::MCQ> = Section::new(questions);

    let grade = quizgen::quiz(1, section);
    println!("{grade}");
}

/*
Answer the following questions:

Question 1
Hello [.....]! Welcome to quizgen.

        A. World
        B. Universe
        C. Galaxy
        D. Planet
 */
