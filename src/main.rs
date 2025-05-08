use quizgen::{mcq, words_api, Question, Section};

fn main() -> anyhow::Result<()> {
    let api = words_api::WordsApi::new(std::env::var("WORDS_API_KEY")?)?;

    let mut questions = Vec::new();
    for word in ["rust"] {
        let response = api.get_examples(word)?;

        if response.examples.is_empty() {
            continue;
        }

        let answer = mcq::Choice::A;
        let mcq = mcq::MCQ::new(
            response.examples[0].clone(),
            ["rust", "go", "swift", "ruby"],
            answer,
        );

        questions.push(Question::new(mcq).answer(Some(answer)));
    }

    let section: Section<mcq::Choice, mcq::MCQ> = Section::new(questions);
    let grade = quizgen::quiz(1, section);
    println!("{grade}");

    Ok(())
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
