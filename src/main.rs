use quizgen::{mcq, Question, Section};
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

#[derive(Debug, Deserialize)]
struct DefinitionResponse {
    pub word: String,
    pub definitions: Vec<Definition>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Definition {
    pub definition: String,
    pub part_of_speech: String,
}

#[derive(Debug, Deserialize)]
struct SynonymResponse {
    pub word: String,
    pub synonyms: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AntonymResponse {
    pub word: String,
    pub antonyms: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ExampleResponse {
    pub word: String,
    pub examples: Vec<String>,
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

    pub fn get_details(&self, word: impl AsRef<str>) -> anyhow::Result<WordResponse> {
        self.get(word, None)
    }

    pub fn get_definitions(&self, word: impl AsRef<str>) -> anyhow::Result<DefinitionResponse> {
        self.get(word, Some(Details::Definitions))
    }

    pub fn get_synonyms(&self, word: impl AsRef<str>) -> anyhow::Result<SynonymResponse> {
        self.get(word, Some(Details::Synonyms))
    }

    pub fn get_antonyms(&self, word: impl AsRef<str>) -> anyhow::Result<AntonymResponse> {
        self.get(word, Some(Details::Antonyms))
    }

    pub fn get_examples(&self, word: impl AsRef<str>) -> anyhow::Result<ExampleResponse> {
        self.get(word, Some(Details::Examples))
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

fn main() -> anyhow::Result<()> {
    let api = WordsApi::new(std::env::var("WORDS_API_KEY")?)?;

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
