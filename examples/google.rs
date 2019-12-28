use std::{convert::Into, thread, time::Duration};
use webview::{geometry::Position, http::Request, Load, WebView};
use winit::{ControlFlow, Event, EventsLoop, Window, WindowEvent};

fn main() {
    let mut events_loop = EventsLoop::new();
    let mut window = Window::new(&events_loop).unwrap();
    let size: (f64, f64) = window.get_inner_size().unwrap().into();

    let mut web_view = WebView::new(Position::new(0.0, 0.0), size.into(), |_, val| {
        eprintln!("invoke called with {}", val);
    });

    web_view.add_to(&mut window);
    web_view.load(Load::Request(
        Request::builder()
            .uri("https://www.google.com")
            .body(())
            .unwrap(),
    ));

    let handle = web_view.handle();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(5));
        handle.dispatch(r#"console.log("hello world")"#).ok();
    });

    events_loop.run_forever(|event| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            println!("The close button was pressed; stopping");
            ControlFlow::Break
        }
        _ => ControlFlow::Continue,
    });
}
