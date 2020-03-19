use reqwest::blocking::Client as HttpClient;
use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_json;

pub struct Client {
    base_url: String,
    token: String,
    http_client: HttpClient,
}

impl Client {
    pub fn new(token: String) -> Result<Self, Box<std::error::Error>> {
        Ok(Client {
            http_client: Client::create_http_client(&token)?,
            token,
            base_url: "https://api.listenbrainz.org".to_owned(),
        })
    }

    pub fn submit_now_playing(&self, track: Track) -> Result<(), Box<dyn std::error::Error>> {
        let request = Request {
            listen_type: ListenType::PlayingNow,
            payload: vec![track],
        };

        let url = format!("{}/1/submit-listens", self.base_url);
        let body = serde_json::to_string(&request).unwrap();

        trace!("Sending request: {:#?}", body);
        let response = self.http_client.post(&url).body(body).send()?;
        trace!("Received: {:#?}", response.text());

        Ok(())
    }

    pub fn submit_listen(&self, track: Track) -> Result<(), Box<dyn std::error::Error>> {
        let request = Request {
            listen_type: ListenType::Single,
            payload: vec![track],
        };

        let url = format!("{}/1/submit-listens", self.base_url);
        let body = serde_json::to_string(&request).unwrap();

        trace!("Sending request: {:#?}", body);
        let response = self.http_client.post(&url).body(body).send()?;
        trace!("Received: {:#?}", response.text());

        Ok(())
    }

    pub fn create_http_client(token: &str) -> Result<HttpClient, Box<std::error::Error>> {
        let token_header = format!("Token {}", token);

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&token_header)?,
        );

        let http_client = HttpClient::builder()
            .user_agent("SCROBBLER LUL")
            .default_headers(headers)
            .build()?;

        Ok(http_client)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub listen_type: ListenType,
    pub payload: Vec<Track>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Track {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listened_at: Option<i64>,
    pub track_metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub artist_name: String,
    pub track_name: String,
    pub release_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_info: Option<AdditionalInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdditionalInfo {
    pub release_mbid: String,
    pub artist_mbids: Vec<String>,
    pub recording_mbid: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ListenType {
    #[serde(rename = "single")]
    Single,

    #[serde(rename = "playing_now")]
    PlayingNow,
}
