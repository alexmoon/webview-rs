pub use http;
pub use json;
pub use raw_window_handle;

pub mod geometry;
mod platform;

pub enum Load<'a> {
    Html { data: &'a str, base: http::Uri },
    Request(http::Request<()>),
}

pub use platform::WebView;
