use crate::{geometry::*, Load};
use cocoa::{
    base::{id, nil},
    foundation::NSString,
};
use core_graphics::geometry::{CGPoint, CGRect, CGSize};
use json::JsonValue;
use objc::{
    class,
    declare::ClassDecl,
    msg_send,
    rc::StrongPtr,
    runtime::{Class, Object, Sel},
    sel, sel_impl,
};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::{ffi::c_void, ptr, slice, str, sync::Once};

pub struct WebView<'a> {
    web_view: StrongPtr,

    #[allow(dead_code)]
    // NOTE: invoke_handler is used by external_invoke() but the compiler can't see it
    invoke_handler: Box<dyn Fn(JsonValue) + 'a>,
}

impl<'a> Drop for WebView<'a> {
    fn drop(&mut self) {
        unsafe {
            let configuration: id = msg_send![*self.web_view, configuration];
            let user_content_controller: id = msg_send![configuration, userContentController];
            let name = StrongPtr::new(NSString::alloc(nil).init_str("invoke"));
            let _: () =
                msg_send![user_content_controller, removeScriptMessageHandlerForName: *name];
        }

        self.remove_from_parent();
    }
}

static mut INVOKE_SCRIPT_MESSAGE_HANDLER_CLASS: *const Class = ptr::null();
static INVOKE_SCRIPT_MESSAGE_HANDLER_CLASS_INIT: Once = Once::new();

extern "C" fn external_invoke<F>(
    this: &Object,
    _cmd: Sel,
    _content_controller: id,
    script_message: id,
) where
    F: FnMut(JsonValue),
{
    let handler: &mut F = unsafe { &mut *(*this.get_ivar::<*mut c_void>("handler") as *mut F) };

    let value = json::parse(unsafe {
        let body: id = msg_send![script_message, body];
        str::from_utf8(slice::from_raw_parts(
            body.UTF8String() as *const u8,
            body.len(),
        ))
        .unwrap()
    })
    .unwrap();

    handler(value);
}

const NSVIEW_WIDTH_SIZABLE: u32 = 2;
const NSVIEW_HEIGHT_SIZABLE: u32 = 16;
const WKUSER_SCRIPT_INJECTION_TIME_AT_DOCUMENT_START: u32 = 0;

impl<'a> WebView<'a> {
    pub fn new<F>(position: Position, size: Size, invoke_handler: F) -> Self
    where
        F: Fn(JsonValue) + 'a,
    {
        let frame = CGRect {
            origin: CGPoint {
                x: position.x,
                y: position.y,
            },
            size: CGSize {
                width: size.width,
                height: size.height,
            },
        };

        let mut invoke_handler = Box::new(invoke_handler);

        let mut web_view = unsafe {
            let configuration = StrongPtr::new(msg_send![class!(WKWebViewConfiguration), new]);

            #[cfg(debug_assertions)]
            {
                let preferences: id = msg_send![*configuration, preferences];
                let _: () = msg_send![preferences, _setDeveloperExtrasEnabled: true];
            }

            INVOKE_SCRIPT_MESSAGE_HANDLER_CLASS_INIT.call_once(|| {
                let mut decl =
                    ClassDecl::new("InvokeScriptMessageHandler", class!(NSObject)).unwrap();
                decl.add_method(
                    sel!(userContentController:didReceiveScriptMessage:),
                    external_invoke::<F> as extern "C" fn(&Object, Sel, id, id),
                );
                decl.add_ivar::<*const c_void>("handler");
                INVOKE_SCRIPT_MESSAGE_HANDLER_CLASS = decl.register() as *const Class;
            });

            let script_message_handler =
                StrongPtr::new(msg_send![INVOKE_SCRIPT_MESSAGE_HANDLER_CLASS, new]);

            (*script_message_handler)
                .as_mut()
                .unwrap()
                .set_ivar("handler", &mut *invoke_handler as *mut _ as *mut c_void);

            let user_content_controller: id = msg_send![*configuration, userContentController];
            let name = StrongPtr::new(NSString::alloc(nil).init_str("invoke"));
            let _: () = msg_send![user_content_controller, addScriptMessageHandler: *script_message_handler name: *name];

            let web_view: id = msg_send![class!(WKWebView), alloc];
            let web_view = StrongPtr::new(
                msg_send![web_view, initWithFrame: frame configuration: *configuration],
            );

            let _: () = msg_send![*web_view, setAutoresizingMask: (NSVIEW_WIDTH_SIZABLE | NSVIEW_HEIGHT_SIZABLE)];

            WebView {
                invoke_handler,
                web_view,
            }
        };

        web_view.inject_script(
            r#"
            window.external = {
                invoke: function(arg) {
                    webkit.messageHandlers.invoke.postMessage(JSON.stringify(arg));
                }
            };
        "#,
        );

        web_view
    }

    pub fn add_to<T: HasRawWindowHandle>(&mut self, window: &mut T) {
        if let RawWindowHandle::MacOS(handle) = window.raw_window_handle() {
            let container = handle.ns_view as id;
            self.remove_from_parent();
            let _: id = unsafe { msg_send![container as id, addSubview: *self.web_view] };
        } else {
            panic!("RawWindowHandle is not MacOS on MacOS");
        }
    }

    pub fn remove_from_parent(&mut self) {
        let _: () = unsafe { msg_send![*self.web_view, removeFromSuperview] };
    }

    pub fn is_hidden(&self) -> bool {
        unsafe { msg_send![*self.web_view, isHidden] }
    }

    pub fn show(&mut self) {
        let _: () = unsafe { msg_send![*self.web_view, setHidden: false] };
    }

    pub fn hide(&mut self) {
        let _: () = unsafe { msg_send![*self.web_view, setHidden: true] };
    }

    pub fn get_position(&self) -> Position {
        let frame: CGRect = unsafe { msg_send![*self.web_view, frame] };
        Position {
            x: frame.origin.x,
            y: frame.origin.y,
        }
    }

    pub fn set_position(&mut self, position: Position) {
        let _: () = unsafe {
            msg_send![*self.web_view, setFrameOrigin: CGPoint { x: position.x, y: position.y }]
        };
    }

    pub fn get_size(&self) -> Size {
        let frame: CGRect = unsafe { msg_send![*self.web_view, frame] };
        Size {
            width: frame.size.width,
            height: frame.size.height,
        }
    }

    pub fn set_size(&mut self, size: Size) {
        let _: () = unsafe {
            msg_send![*self.web_view, setFrameSize: CGSize { width: size.width, height: size.height }]
        };
    }

    pub fn load(&mut self, request: Load) {
        match request {
            Load::Html { data, base } => unsafe {
                let html = StrongPtr::new(NSString::alloc(nil).init_str(data));
                let url = StrongPtr::new(
                    msg_send![class!(NSURL), URLWithString: NSString::alloc(nil).init_str(&base.to_string())],
                );
                let _: id = msg_send![*self.web_view, loadHTMLString: *html baseURL: *url];
            },
            Load::Request(req) => unsafe {
                let url = StrongPtr::new(
                    msg_send![class!(NSURL), URLWithString: NSString::alloc(nil).init_str(&req.uri().to_string())],
                );
                let request =
                    StrongPtr::new(msg_send![class!(NSMutableURLRequest), requestWithURL: *url]);
                let method = StrongPtr::new(NSString::alloc(nil).init_str(req.method().as_str()));
                let _: () = msg_send![*request, setHTTPMethod: *method];

                const ASCII_ENCODING: usize = 1;

                for (name, value) in req.headers() {
                    let name = StrongPtr::new(NSString::alloc(nil).init_str(name.as_str()));
                    let value = value.as_bytes();
                    let value = StrongPtr::new(
                        msg_send![NSString::alloc(nil), initWithBytes: value.as_ptr() length: value.len() encoding: ASCII_ENCODING],
                    );
                    let _: () = msg_send![*request, setValue: *value forHTTPHeaderField: *name];
                }

                let _: id = msg_send![*self.web_view, loadRequest: *request];
            },
        }
    }

    pub fn eval(&mut self, script: &str) {
        unsafe {
            let script = StrongPtr::new(NSString::alloc(nil).init_str(script));
            let _: () = msg_send![*self.web_view, evaluateJavaScript: script completionHandler: ptr::null::<c_void>() ];
        }
    }

    pub fn inject_script(&mut self, script: &str) {
        unsafe {
            let configuration: id = msg_send![*self.web_view, configuration];
            let user_content_controller: id = msg_send![configuration, userContentController];
            let source = StrongPtr::new(NSString::alloc(nil).init_str(script));
            let user_script: id = msg_send![class!(WKUserScript), alloc];
            let user_script = StrongPtr::new(
                msg_send![user_script, initWithSource: *source injectionTime: WKUSER_SCRIPT_INJECTION_TIME_AT_DOCUMENT_START forMainFrameOnly: true],
            );

            let _: () = msg_send![user_content_controller, addUserScript: *user_script];
        }
    }
}
