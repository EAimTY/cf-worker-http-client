#![doc = include_str!("../README.md")]

mod util;

pub mod agent;
pub mod request;
pub mod response;

pub use crate::{agent::Agent, request::Request, response::Response};
