pub use http;
pub use raw_window_handle;

use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

pub mod geometry;
mod platform;

use geometry::{Position, Size};

pub enum Load<'a> {
    Html { data: &'a str, base: http::Uri },
    Request(http::Request<()>),
}

pub struct WebView<'a> {
    inner: Box<platform::WebView<'a>>,
}

pub use platform::Handle;

impl<'a> WebView<'a> {
    pub fn new<F>(position: Position, size: Size, mut invoke_handler: F) -> Self
    where
        F: FnMut(&mut Self, &str) + 'a,
    {
        let mut inner = Box::new(platform::WebView::new(move |inner, val| {
            let mut webview = ManuallyDrop::new(WebView {
                inner: unsafe { Box::from_raw(inner) },
            });
            invoke_handler(&mut webview, val);
        }));

        inner.init(position, size);

        WebView { inner }
    }
}

impl<'a> Deref for WebView<'a> {
    type Target = platform::WebView<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> DerefMut for WebView<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
