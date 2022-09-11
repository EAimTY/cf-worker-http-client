use crate::{
    request::{Get, Post},
    Request,
};
use cookie_store::CookieStore;
use parking_lot::Mutex;
use std::sync::Arc;
use url::Url;
use worker::Method;

#[derive(Default)]
pub struct Agent {
    cookie_store: Arc<Mutex<CookieStore>>,
}

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
        Request::new(url, method, self.cookie_store.clone(), _marker)
    }
}
