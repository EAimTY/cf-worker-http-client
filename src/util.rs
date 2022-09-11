use cookie::Cookie;
use cookie_store::CookieStore;
use std::{slice, str};
use url::Url;

pub(crate) fn add_response_cookies(store: &mut CookieStore, cookies: &str, url: &Url) {
    let mut cookies = cookies.split_inclusive(',').peekable();
    let mut pending = None;
    let mut curr_part = cookies.next();
    let mut next_part = cookies.peek();

    while let Some(next_cookie) = next_part {
        if let Ok(next_cookie) = Cookie::parse(*next_cookie) {
            let new =
                pending.or_else(|| curr_part.and_then(|curr_part| Cookie::parse(curr_part).ok()));

            let _ = store.insert_raw(&new.unwrap(), url);
            pending = Some(next_cookie);
        } else {
            let ptr = curr_part.unwrap().as_ptr();
            let len = curr_part.unwrap().len() + next_part.unwrap().len();

            let curr_cookie = unsafe { str::from_utf8_unchecked(slice::from_raw_parts(ptr, len)) };

            if let Ok(new) = Cookie::parse(curr_cookie) {
                let _ = store.insert_raw(&new, url);
            }

            cookies.next();
        }

        curr_part = cookies.next();
        next_part = cookies.peek();
    }

    if let Some(new) = curr_part.and_then(|cookie| Cookie::parse(cookie).ok()) {
        let _ = store.insert_raw(&new, url);
    }
}
