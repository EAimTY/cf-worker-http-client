use crate::{Agent, Response};
use url::{form_urlencoded::Serializer, Url};
use worker::{
    wasm_bindgen::JsValue,
    worker_sys::{
        Request as EdgeRequest, RequestInit as EdgeRequestInit,
        RequestRedirect as EdgeRequestRedirect,
    },
    Error, Fetch, Headers, Method, Request as WorkerRequest, Response as WorkerResponse,
};

pub struct Get;
pub struct Post;

pub struct Request<M> {
    agent: Agent,
    url: Url,
    method: Method,
    headers: Headers,
    _marker: M,
}

impl<M> Request<M> {
    pub(crate) fn new(agent: Agent, url: Url, method: Method, _marker: M) -> Self {
        let mut headers = Headers::new();
        let cookies = agent.get_request_cookies(&url);
        headers.set("Cookie", &cookies).unwrap();

        Self {
            agent,
            url,
            method,
            headers,
            _marker,
        }
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn headers(&mut self) -> &mut Headers {
        &mut self.headers
    }

    async fn do_call(self, body: Option<&JsValue>) -> Result<Response, Error> {
        let mut resp = self.get_response_inner(body).await?;
        let mut url = self.url;

        loop {
            if resp.status_code() != 301 && resp.status_code() != 302 {
                if let Some(cookies) = resp.headers().get("Set-Cookie")? {
                    self.agent.store_response_cookies(&url, &cookies);
                }

                return Ok(Response::new(resp, url));
            } else if let Some(redir_url) = resp
                .headers()
                .get("Location")?
                .and_then(|location| url.join(&location).ok())
            {
                if let Some(cookies) = resp.headers().get("Set-Cookie")? {
                    self.agent.store_response_cookies(&redir_url, &cookies);
                }

                let req = self.agent.get(redir_url);
                resp = req.get_response_inner(None).await?;
                url = req.url;
            } else {
                if let Some(cookies) = resp.headers().get("Set-Cookie")? {
                    self.agent.store_response_cookies(&url, &cookies);
                }

                return Ok(Response::new(resp, url));
            }
        }
    }

    async fn get_response_inner(&self, body: Option<&JsValue>) -> Result<WorkerResponse, Error> {
        let mut init = EdgeRequestInit::new();

        init.method(self.method.as_ref())
            .headers(&self.headers.0)
            .redirect(EdgeRequestRedirect::Manual)
            .body(body);

        let req = EdgeRequest::new_with_str_and_init(self.url.as_str(), &init)?;
        Fetch::Request(WorkerRequest::from(req)).send().await
    }
}

impl Request<Get> {
    pub async fn call(self) -> Result<Response, Error> {
        self.do_call(None).await
    }
}

impl Request<Post> {
    pub async fn send_form(
        mut self,
        form: impl IntoIterator<Item = (impl AsRef<str>, impl AsRef<str>)>,
    ) -> Result<Response, Error> {
        self.headers
            .set("Content-Type", "application/x-www-form-urlencoded")?;

        let form = JsValue::from(Serializer::new(String::new()).extend_pairs(form).finish());

        self.do_call(Some(&form)).await
    }
}
