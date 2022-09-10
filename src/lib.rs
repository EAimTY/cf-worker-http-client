use cookie::Cookie;
use cookie_store::CookieStore;
use parking_lot::Mutex;
use std::{fmt::Write, slice, str, sync::Arc};
use url::{form_urlencoded::Serializer, Url};
use worker::{Fetch, Method, Request as WorkerRequest, Response as WorkerResponse, Result};

pub struct Agent {
    cookies: Arc<Mutex<CookieStore>>,
}

impl Agent {
    pub fn new() -> Self {
        Self {
            cookies: Arc::new(Mutex::new(CookieStore::default())),
        }
    }

    pub fn get(&self, url: &Url) -> Result<Request<Get>> {
        Ok(Request::new(
            self.init_request(&url, Method::Get)?,
            self.cookies.clone(),
            Get,
        ))
    }

    pub fn post(&self, url: &Url) -> Result<Request<Post>> {
        Ok(Request::new(
            self.init_request(&url, Method::Post)?,
            self.cookies.clone(),
            Post,
        ))
    }

    fn init_request(&self, url: &Url, method: Method) -> Result<WorkerRequest> {
        let mut req = WorkerRequest::new(url.as_str(), method)?;
        let mut cookies = String::new();

        for (name, value) in self.cookies.lock().get_request_values(&url) {
            write!(&mut cookies, "{name}={value}; ").unwrap();
        }

        req.headers_mut()?.set("Cookie", &cookies)?;

        Ok(req)
    }
}

pub struct Request<M> {
    inner: WorkerRequest,
    cookies: Arc<Mutex<CookieStore>>,
    _method: M,
}

impl<M> Request<M> {
    fn new(inner: WorkerRequest, cookies: Arc<Mutex<CookieStore>>, method: M) -> Self {
        Self {
            inner,
            cookies,
            _method: method,
        }
    }

    async fn do_call(self) -> Result<WorkerResponse> {
        let url = self.inner.url()?;
        let resp = Fetch::Request(self.inner).send().await?;

        if let Some(cookies) = resp.headers().get("Set-Cookie")? {
            Self::add_response_cookies(&mut self.cookies.lock(), &cookies, &url);
        }

        Ok(resp)
    }

    fn add_response_cookies(store: &mut CookieStore, cookies: &str, url: &Url) {
        let mut cookies = cookies.split_inclusive(',').peekable();
        let mut pending = None;
        let mut curr_part = cookies.next();
        let mut next_part = cookies.peek();

        while let Some(next_cookie) = next_part {
            if let Ok(next_cookie) = Cookie::parse(*next_cookie) {
                let new = pending.or_else(|| {
                    curr_part
                        .map(|curr_part| Cookie::parse(curr_part).ok())
                        .flatten()
                });

                let _ = store.insert_raw(&new.unwrap(), url);
                pending = Some(next_cookie);
            } else {
                let ptr = curr_part.unwrap().as_ptr();
                let len = curr_part.unwrap().len() + next_part.unwrap().len();

                let curr_cookie =
                    unsafe { str::from_utf8_unchecked(slice::from_raw_parts(ptr, len)) };

                if let Ok(new) = Cookie::parse(curr_cookie) {
                    let _ = store.insert_raw(&new, url);
                }

                cookies.next();
            }

            curr_part = cookies.next();
            next_part = cookies.peek();
        }

        if let Some(new) = curr_part.map(|cookie| Cookie::parse(cookie).ok()).flatten() {
            let _ = store.insert_raw(&new, url);
        }
    }
}

impl Request<Get> {
    pub async fn call(self) -> Result<Response> {
        Ok(Response(self.do_call().await?))
    }
}

impl Request<Post> {
    pub async fn send_form(
        mut self,
        form: impl IntoIterator<Item = (impl AsRef<str>, impl AsRef<str>)>,
    ) -> Result<Response> {
        self.inner
            .headers_mut()?
            .set("Content-Type", "application/x-www-form-urlencoded")?;

        let _encoded = Serializer::new(String::new()).extend_pairs(form).finish();

        Ok(Response(self.do_call().await?))
    }
}

pub struct Response(WorkerResponse);

pub struct Get;
pub struct Post;
