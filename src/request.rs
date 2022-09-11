use crate::{util, Response};
use cookie_store::CookieStore;
use parking_lot::Mutex;
use std::{fmt::Write, sync::Arc};
use url::{form_urlencoded::Serializer, Url};
use worker::{
    wasm_bindgen::JsValue,
    worker_sys::{
        Request as EdgeRequest, RequestInit as EdgeRequestInit,
        RequestRedirect as EdgeRequestRedirect,
    },
    Error, Fetch, Headers, Method, Request as WorkerRequest, RequestRedirect,
};

pub struct Get;
pub struct Post;

pub struct Request<M> {
    url: Url,
    method: Method,
    headers: Headers,
    redirect: RequestRedirect,
    cookie_store: Arc<Mutex<CookieStore>>,
    _marker: M,
}

impl<M> Request<M> {
    pub(crate) fn new(
        url: Url,
        method: Method,
        cookie_store: Arc<Mutex<CookieStore>>,
        _marker: M,
    ) -> Self {
        let mut headers = Headers::new();
        let mut cookies = String::new();

        for (name, value) in cookie_store.lock().get_request_values(&url) {
            write!(&mut cookies, "{name}={value}; ").unwrap();
        }

        headers.set("Cookie", &cookies).unwrap();

        Self {
            url,
            method,
            headers,
            redirect: RequestRedirect::Follow,
            cookie_store,
            _marker,
        }
    }

    pub fn headers(&mut self) -> &mut Headers {
        &mut self.headers
    }

    async fn do_call(self, body: Option<&JsValue>) -> Result<Response, Error> {
        let mut init = EdgeRequestInit::new();

        init.method(self.method.as_ref())
            .headers(&self.headers.0)
            .redirect(EdgeRequestRedirect::from(self.redirect))
            .body(body);

        let req = EdgeRequest::new_with_str_and_init(self.url.as_str(), &init)?;
        let resp = Fetch::Request(WorkerRequest::from(req)).send().await?;

        if let Some(cookies) = resp.headers().get("Set-Cookie")? {
            util::add_response_cookies(&mut self.cookie_store.lock(), &cookies, &self.url);
        }

        Ok(Response::new(resp))
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
