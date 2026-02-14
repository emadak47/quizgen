use reqwest::blocking::{Client, Response};
use serde::{de::DeserializeOwned, Deserialize};
use url::Url;

use super::english::{
    AntonymResponse, DefinitionResponse, Details, EnglishApi, ExampleResponse, SynonymResponse,
};

#[derive(Debug, Deserialize)]
pub struct WordResponse {
    pub word: String,
    pub results: Vec<WordDetails>,
    pub frequency: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordDetails {
    pub definition: String,
    pub part_of_speech: String,
    pub derivation: Option<Vec<String>>,
    pub synonyms: Option<Vec<String>>,
    pub similar_to: Option<Vec<String>>,
    pub type_of: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct DefinitionResponseTemp {
    word: String,
    definitions: Vec<Definition>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Definition {
    definition: String,
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

    fn handle_response<T: DeserializeOwned>(&self, response: Response) -> anyhow::Result<T> {
        let status = response.status();

        if status.is_success() {
            response.json().map_err(|e| e.into())
        } else {
            anyhow::bail!("HTTP error {} {}", status, response.text()?);
        }
    }
}

impl EnglishApi for WordsApi {
    fn get_definitions(&self, word: &str) -> anyhow::Result<DefinitionResponse> {
        let resp: DefinitionResponseTemp = self.get(word, Some(Details::Definitions))?;

        Ok(DefinitionResponse {
            word: resp.word,
            definitions: resp
                .definitions
                .into_iter()
                .map(|def| def.definition)
                .collect(),
        })
    }

    fn get_synonyms(&self, word: &str) -> anyhow::Result<SynonymResponse> {
        self.get(word, Some(Details::Synonyms))
    }

    fn get_antonyms(&self, word: &str) -> anyhow::Result<AntonymResponse> {
        self.get(word, Some(Details::Antonyms))
    }

    fn get_examples(&self, word: &str) -> anyhow::Result<ExampleResponse> {
        self.get(word, Some(Details::Examples))
    }
}
