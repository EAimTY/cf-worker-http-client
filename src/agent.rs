use crate::{
    request::{Get, Post},
    Request,
};
use cookie::Cookie;
use cookie_store::CookieStore;
use parking_lot::Mutex;
use std::{fmt::Write, slice, str, sync::Arc};
use url::Url;
use worker::Method;

#[derive(Clone, Default)]
pub struct Agent(Arc<AgentInner>);

impl Agent {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get(&self, url: Url) -> Request<Get> {
        self.init_request(url, Method::Get, Get)
    }

    pub fn post(&self, url: Url) -> Request<Post> {
        self.init_request(url, Method::Post, Post)
    }

    fn init_request<M>(&self, url: Url, method: Method, _marker: M) -> Request<M> {
        Request::new(self.clone(), url, method, _marker)
    }

    pub(crate) fn get_request_cookies(&self, url: &Url) -> String {
        let mut cookies = String::new();

        for (name, value) in self.0.cookie_store.lock().get_request_values(url) {
            write!(&mut cookies, "{name}={value}; ").unwrap();
        }

        cookies
    }

    pub(crate) fn store_response_cookies(&self, url: &Url, cookies: &str) {
        let mut cookies = cookies.split_inclusive(',').peekable();
        let mut pending = None;
        let mut curr_part = cookies.next();
        let mut next_part = cookies.peek();

        while let Some(next_cookie) = next_part {
            if let Ok(next_cookie) = Cookie::parse(*next_cookie) {
                let new = pending
                    .or_else(|| curr_part.and_then(|curr_part| Cookie::parse(curr_part).ok()));

                let _ = self.0.cookie_store.lock().insert_raw(&new.unwrap(), url);
                pending = Some(next_cookie);
            } else {
                let ptr = curr_part.unwrap().as_ptr();
                let len = curr_part.unwrap().len() + next_part.unwrap().len();

                let curr_cookie =
                    unsafe { str::from_utf8_unchecked(slice::from_raw_parts(ptr, len)) };

                if let Ok(new) = Cookie::parse(curr_cookie) {
                    let _ = self.0.cookie_store.lock().insert_raw(&new, url);
                }

                cookies.next();
            }

            curr_part = cookies.next();
            next_part = cookies.peek();
        }

        if let Some(new) = curr_part.and_then(|cookie| Cookie::parse(cookie).ok()) {
            let _ = self.0.cookie_store.lock().insert_raw(&new, url);
        }
    }
}

#[derive(Default)]
struct AgentInner {
    cookie_store: Mutex<CookieStore>,
}
