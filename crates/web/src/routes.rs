use std::str::FromStr;

use askama::Template;
use askama_web::WebTemplate;
use axum::extract::{Path, State};
use axum::response::Redirect;
use axum::Form;
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};

use quizgen_core::english::{Details, EnglishQuiz};
use quizgen_core::mcq::Choice;
use quizgen_core::webster::WebsterApi;
use quizgen_core::words_api::WordsApi;
use quizgen_core::GradedQuiz;

use crate::error::WebError;
use crate::session::QuizSession;
use crate::AppState;

const SESSION_COOKIE: &str = "quizgen_session";

/// Minimal HTML escape — sufficient for plain-text dictionary content.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn get_session_id(cookies: &Cookies) -> Option<String> {
    cookies.get(SESSION_COOKIE).map(|c| c.value().to_string())
}

#[derive(Template, WebTemplate)]
#[template(path = "index.html")]
pub struct IndexTemplate;

pub async fn index() -> IndexTemplate {
    IndexTemplate
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
    let kind =
        Details::from_str(&form.quiz_type).map_err(|e| WebError::BadRequest(e.to_string()))?;

    let words_api =
        WordsApi::new(&state.words_api_key).map_err(|e| WebError::Internal(e.to_string()))?;
    let webster_api = WebsterApi::new(&state.collegiate_key, &state.thesaurus_key)
        .map_err(|e| WebError::Internal(e.to_string()))?;

    let mut english_quiz = EnglishQuiz::new(
        [Box::new(words_api), Box::new(webster_api)],
        &state.source_dir,
        kind,
    )?;

    let questions = english_quiz.gen_n_mcqs::<4>(form.length).await?;

    if questions.is_empty() {
        return Err(WebError::ServiceUnavailable);
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

#[derive(Template, WebTemplate)]
#[template(path = "question.html")]
pub struct QuestionTemplate {
    current: usize,           // for form action URL, film strip loop
    total: usize,             // for film strip range
    current_display: String,  // "03" — zero-padded for display
    total_display: String,    // "10" — zero-padded for display
    statement: String,        // contains HTML <span> for blank
    choices: Vec<(char, String)>,
}

pub async fn show_question(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(n): Path<usize>,
) -> Result<QuestionTemplate, WebError> {
    let session_id = get_session_id(&cookies).ok_or(WebError::NoSession)?;

    let question_temp = state
        .store
        .get(&session_id, |session| {
            if n == 0 || n > session.questions.len() {
                return None;
            }
            let q = &session.questions[n - 1];
            let solution_word = &q.choices()[q.solution() as usize];
            let escaped_stmt = html_escape(q.statement());
            let escaped_word = html_escape(solution_word);
            let statement = escaped_stmt.replacen(&escaped_word, "<span class=\"q-blank\"></span>", 1);
            let choices: Vec<(char, String)> = q
                .choices()
                .iter()
                .enumerate()
                .map(|(i, c)| ((b'a' + i as u8) as char, c.clone()))
                .collect();
            let total = session.questions.len();
            Some(QuestionTemplate {
                current: n,
                total,
                current_display: format!("{:02}", n),
                total_display: format!("{:02}", total),
                statement,
                choices,
            })
        })
        .await
        .ok_or(WebError::NoSession)?
        .ok_or(WebError::NotFound)?;

    Ok(question_temp)
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

    let result_temp = state
        .store
        .get(&session_id, |session| {
            let graded = GradedQuiz::new(
                &session.questions,
                &session.answers,
                session.started_at.elapsed(),
            );
            let results: Vec<QuestionResult> = graded
                .iter()
                .map(|g| QuestionResult {
                    correct: g.correct,
                    correct_answer: g.correct_answer.to_owned(),
                    your_answer: g.your_answer.unwrap_or("\u{2014}").to_owned(),
                })
                .collect();
            ResultsTemplate {
                time_taken: format!("{:.1}s", graded.elapsed.as_secs_f64()),
                score: format!("{:.1}%", graded.score()),
                results,
            }
        })
        .await
        .ok_or(WebError::NoSession)?;

    // Clean up session
    state.store.remove(&session_id).await;
    cookies.remove(Cookie::from(SESSION_COOKIE));

    Ok(result_temp)
}
