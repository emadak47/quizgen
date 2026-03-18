use quizgen_core::mcq::{Choice, Mcq};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;

pub type SessionId = String;

pub struct QuizSession {
    pub questions: Vec<Mcq<4>>,
    pub answers: Vec<Option<Choice>>,
    pub current: usize,
    pub started_at: Instant,
}

impl QuizSession {
    pub fn new(questions: Vec<Mcq<4>>) -> Self {
        let len = questions.len();
        Self {
            questions,
            answers: vec![None; len],
            current: 0,
            started_at: Instant::now(),
        }
    }
}

#[derive(Clone)]
pub struct SessionStore {
    sessions: Arc<RwLock<HashMap<SessionId, QuizSession>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create(&self, session: QuizSession) -> SessionId {
        let id = Uuid::new_v4().to_string();
        self.sessions.write().await.insert(id.clone(), session);
        id
    }

    pub async fn get<F, R>(&self, id: &str, f: F) -> Option<R>
    where
        F: FnOnce(&QuizSession) -> R,
    {
        self.sessions.read().await.get(id).map(f)
    }

    pub async fn get_mut<F, R>(&self, id: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut QuizSession) -> R,
    {
        self.sessions.write().await.get_mut(id).map(f)
    }

    pub async fn remove(&self, id: &str) {
        self.sessions.write().await.remove(id);
    }
}
