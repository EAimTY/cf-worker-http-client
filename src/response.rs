use worker::{Error, Headers, Response as WorkerResponse};

pub struct Response {
    inner: WorkerResponse,
}

impl Response {
    pub(crate) fn new(resp: WorkerResponse) -> Self {
        Self { inner: resp }
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
