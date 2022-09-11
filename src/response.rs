use url::Url;
use worker::{Error, Headers, Response as WorkerResponse};

pub struct Response {
    inner: WorkerResponse,
    url: Url,
}

impl Response {
    pub(crate) fn new(resp: WorkerResponse, url: Url) -> Self {
        Self { inner: resp, url }
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn headers(&self) -> &Headers {
        self.inner.headers()
    }

    pub fn status_code(&self) -> u16 {
        self.inner.status_code()
    }

    pub async fn bytes(&mut self) -> Result<Vec<u8>, Error> {
        self.inner.bytes().await
    }

    pub async fn text(&mut self) -> Result<String, Error> {
        self.inner.text().await
    }
}
