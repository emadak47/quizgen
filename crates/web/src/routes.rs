use std::str::FromStr;

use askama::Template;
use askama_web::WebTemplate;
use axum::extract::{Path, State};
use axum::response::Redirect;
use axum::Form;
use quizgen_core::english::{Details, EnglishQuiz};
use quizgen_core::mcq::{Choice, Mcq};
use quizgen_core::webster::WebsterApi;
use quizgen_core::words_api::WordsApi;
use quizgen_core::QuizgenError;
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};

use crate::error::WebError;
use crate::session::QuizSession;
use crate::AppState;

const SESSION_COOKIE: &str = "quizgen_session";

fn get_session_id(cookies: &Cookies) -> Option<String> {
    cookies.get(SESSION_COOKIE).map(|c| c.value().to_string())
}

#[derive(Deserialize)]
pub struct StartForm {
    quiz_type: String,
    length: usize,
}

#[axum::debug_handler]
pub async fn start_quiz(
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<StartForm>,
) -> Result<Redirect, WebError> {
    let kind = Details::from_str(&form.quiz_type).map_err(|e| WebError::Internal(e.to_string()))?;

    let words_api_key = std::env::var("WORDS_API_KEY")
        .map_err(|_| WebError::Internal("Missing WORDS_API_KEY".into()))?;
    let collegiate_key = std::env::var("COLLEGIATE_API_KEY")
        .map_err(|_| WebError::Internal("Missing COLLEGIATE_API_KEY".into()))?;
    let thesaurus_key = std::env::var("THESAURUS_API_KEY")
        .map_err(|_| WebError::Internal("Missing THESAURUS_API_KEY".into()))?;

    let words_api = WordsApi::new(words_api_key).map_err(|e| WebError::Internal(e.to_string()))?;
    let webster_api = WebsterApi::new(collegiate_key, thesaurus_key)
        .map_err(|e| WebError::Internal(e.to_string()))?;

    let mut quiz = EnglishQuiz::new(
        [Box::new(words_api), Box::new(webster_api)],
        &state.source_dir,
        kind,
    )?;

    // Generate all questions up front
    let mut questions: Vec<Mcq<4>> = Vec::with_capacity(form.length);
    while questions.len() < form.length {
        match quiz.gen_rand_mcq::<4>().await {
            Some(Ok(q)) => questions.push(q),
            Some(Err(QuizgenError::DataError)) => continue,
            Some(Err(e)) => return Err(e.into()),
            None => break,
        }
    }

    if questions.is_empty() {
        return Err(WebError::Internal(
            "Could not generate any questions".into(),
        ));
    }

    let session = QuizSession::new(questions);
    let session_id = state.store.create(session).await;

    let mut cookie = Cookie::new(SESSION_COOKIE, session_id);
    cookie.set_http_only(true);
    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
    cookie.set_path("/");
    cookie.set_max_age(tower_cookies::cookie::time::Duration::hours(1));
    cookies.add(cookie);

    Ok(Redirect::to("/quiz/question/1"))
}

struct QuestionData {
    statement: String,
    choices: Vec<(char, String)>,
    total: usize,
}

#[derive(Template, WebTemplate)]
#[template(path = "question.html")]
pub struct QuestionTemplate {
    current: usize,
    total: usize,
    statement: String,
    choices: Vec<(char, String)>,
}

pub async fn show_question(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(n): Path<usize>,
) -> Result<QuestionTemplate, WebError> {
    let session_id = get_session_id(&cookies).ok_or(WebError::NoSession)?;

    let result = state
        .store
        .get(&session_id, |session| {
            if n == 0 || n > session.questions.len() {
                return None;
            }
            let q = &session.questions[n - 1];
            let solution_word = &q.choices()[q.solution() as usize];
            let statement = q.statement().replacen(solution_word, "[.....]", 1);
            let choices: Vec<(char, String)> = q
                .choices()
                .iter()
                .enumerate()
                .map(|(i, c)| ((b'A' + i as u8) as char, c.clone()))
                .collect();
            Some(QuestionData {
                statement,
                choices,
                total: session.questions.len(),
            })
        })
        .await
        .ok_or(WebError::NoSession)?;

    let data = result.ok_or_else(|| WebError::Internal("Invalid question number".into()))?;

    Ok(QuestionTemplate {
        current: n,
        total: data.total,
        statement: data.statement,
        choices: data.choices,
    })
}

#[derive(Deserialize)]
pub struct AnswerForm {
    answer: usize,
}

pub async fn submit_answer(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(n): Path<usize>,
    Form(form): Form<AnswerForm>,
) -> Result<Redirect, WebError> {
    let session_id = get_session_id(&cookies).ok_or(WebError::NoSession)?;

    let total = state
        .store
        .get_mut(&session_id, |session| {
            if n > 0 && n <= session.questions.len() {
                let choice = Choice::try_from(form.answer).ok();
                session.answers[n - 1] = choice;
            }
            session.questions.len()
        })
        .await
        .ok_or(WebError::NoSession)?;

    if n < total {
        Ok(Redirect::to(&format!("/quiz/question/{}", n + 1)))
    } else {
        Ok(Redirect::to("/quiz/results"))
    }
}

pub struct QuestionResult {
    pub correct: bool,
    pub correct_answer: String,
    pub your_answer: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "results.html")]
pub struct ResultsTemplate {
    time_taken: String,
    score: String,
    results: Vec<QuestionResult>,
}

pub async fn show_results(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<ResultsTemplate, WebError> {
    let session_id = get_session_id(&cookies).ok_or(WebError::NoSession)?;

    let data = state
        .store
        .get(&session_id, |session| {
            let elapsed = session.started_at.elapsed();
            let total = session.questions.len();
            let mut correct_count = 0usize;

            let results: Vec<QuestionResult> = session
                .questions
                .iter()
                .zip(session.answers.iter())
                .map(|(q, a)| {
                    let correct_choice = q.solution();
                    let correct_answer = q.choices()[correct_choice as usize].clone();
                    let is_correct = a.is_some_and(|a| a == correct_choice);
                    if is_correct {
                        correct_count += 1;
                    }
                    let your_answer = match a {
                        Some(c) => q.choices()[*c as usize].clone(),
                        None => "\u{2014}".to_string(),
                    };
                    QuestionResult {
                        correct: is_correct,
                        correct_answer,
                        your_answer,
                    }
                })
                .collect();

            let score = if total > 0 {
                format!("{:.1}%", correct_count as f64 / total as f64 * 100.0)
            } else {
                "0.0%".to_string()
            };

            let time_taken = format!("{:.1}s", elapsed.as_secs_f64());
            (time_taken, score, results)
        })
        .await
        .ok_or(WebError::NoSession)?;

    // Clean up session
    state.store.remove(&session_id).await;
    cookies.remove(Cookie::from(SESSION_COOKIE));

    let (time_taken, score, results) = data;
    Ok(ResultsTemplate {
        time_taken,
        score,
        results,
    })
}
