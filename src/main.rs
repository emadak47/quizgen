use std::str::FromStr;

use quizgen::{mcq, words_api, Question, QuizMode, Section};
use rand::prelude::*;

fn main() -> anyhow::Result<()> {
    let api = words_api::WordsApi::new(std::env::var("WORDS_API_KEY")?)?;

    let mut questions = Vec::new();
    for word in ["rust", "sad"] {
        let examples_resp = api.get_examples(word)?;
        let synonyms_resp = api.get_synonyms(word)?;

        if examples_resp.examples.is_empty()
            || synonyms_resp.synonyms.len() < 3
            || synonyms_resp.word != examples_resp.word
        {
            continue;
        }

        let mut synonyms: Vec<String> = synonyms_resp
            .synonyms
            .into_iter()
            .filter(|s| s.to_lowercase() != word.to_lowercase())
            .collect();
        synonyms.shuffle(&mut rand::rng());

        let mut choices: [String; 4] = synonyms
            .into_iter()
            .take(3)
            .chain([word.to_string()])
            .collect::<Vec<_>>()
            .try_into()
            .expect("Should have exactly 4 choices");
        choices.shuffle(&mut rand::rng());

        let correct_index = choices
            .iter()
            .position(|c| c.to_lowercase() == word.to_lowercase())
            .expect("Correct choice is present");

        let answer = mcq::Choice::try_from(correct_index).expect("Choice is valid");
        let example = examples_resp
            .examples
            .choose(&mut rand::rng())
            .expect("Examples are not empty");
        let mcq = mcq::MCQ::new(example, choices, answer);

        questions.push(Question::new(mcq).answer(Some(answer)));
    }

    let num_questions = questions.len();
    let section: Section<mcq::Choice, mcq::MCQ> = Section::new(questions);

    println!("Choose quiz mode:");
    println!("1. Interactive (one question at a time)");
    println!("2. Batch (all questions at once)");
    print!("Enter your choice (1 or 2): ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    let mode = QuizMode::from_str(input.trim()).expect("Invalid choice.");

    let grade = quizgen::quiz(num_questions, section, mode);
    println!("Your final grade: {grade:.1}%");

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
