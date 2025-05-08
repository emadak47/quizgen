use reqwest::blocking::{Client, Response};
use serde::{de::DeserializeOwned, Deserialize};
use url::Url;

#[derive(Debug, Deserialize)]
struct WordResponse {
    pub word: String,
    pub results: Vec<WordDetails>,
    pub frequency: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WordDetails {
    pub definition: String,
    pub part_of_speech: String,
    pub derivation: Option<Vec<String>>,
    pub synonyms: Option<Vec<String>>,
    pub similar_to: Option<Vec<String>>,
    pub type_of: Option<Vec<String>>,
}

enum Details {
    Definitions,
    Synonyms,
    Antonyms,
    Examples,
}

impl std::fmt::Display for Details {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Details::Definitions => write!(f, "definitions"),
            Details::Synonyms => write!(f, "synonyms"),
            Details::Antonyms => write!(f, "antonyms"),
            Details::Examples => write!(f, "examples"),
        }
    }
}

pub struct WordsApi {
    base_url: Url,
    api_key: String,
    client: Client,
}

impl WordsApi {
    pub fn new(api_key: impl Into<String>) -> anyhow::Result<Self> {
        Ok(Self {
            base_url: Url::parse("https://wordsapiv1.p.rapidapi.com/")?,
            api_key: api_key.into(),
            client: Client::new(),
        })
    }

    fn get<T: DeserializeOwned>(
        &self,
        word: impl AsRef<str>,
        details: Option<Details>,
    ) -> anyhow::Result<T> {
        let mut url = self.base_url.clone();
        let path = if let Some(endpoint) = details {
            &format!("words/{}/{endpoint}", word.as_ref())
        } else {
            &format!("words/{}", word.as_ref())
        };
        url.set_path(path);

        let response = self
            .client
            .get(url)
            .header("x-rapidapi-host", "wordsapiv1.p.rapidapi.com")
            .header("x-rapidapi-key", &self.api_key)
            .send()?;

        self.handle_response(response)
    }

    fn handle_response<T: DeserializeOwned>(&self, response: Response) -> anyhow::Result<T> {
        let status = response.status();

        if status.is_success() {
            response.json().map_err(|e| e.into())
        } else {
            anyhow::bail!("HTTP error {} {}", status, response.text()?);
        }
    }
}

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
