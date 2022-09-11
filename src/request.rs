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

    pub fn headers(&mut self) -> &mut Headers {
        &mut self.headers
    }

    async fn do_call(self, body: Option<&JsValue>) -> Result<Response, Error> {
        let mut resp = self.get_response_inner(body).await?;

        loop {
            if resp.status_code() != 301 && resp.status_code() != 302 {
                if let Some(cookies) = resp.headers().get("Set-Cookie")? {
                    self.agent.store_response_cookies(&self.url, &cookies);
                }

                break Ok(Response::new(resp));
            } else if let Some(redir_url) = resp
                .headers()
                .get("Location")?
                .and_then(|location| Url::parse(&location).ok())
            {
                if let Some(cookies) = resp.headers().get("Set-Cookie")? {
                    self.agent.store_response_cookies(&redir_url, &cookies);
                }

                resp = self.agent.get(redir_url).get_response_inner(None).await?;
            } else {
                if let Some(cookies) = resp.headers().get("Set-Cookie")? {
                    self.agent.store_response_cookies(&self.url, &cookies);
                }

                break Ok(Response::new(resp));
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
