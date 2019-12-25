use std::convert::Into;
use webview::{geometry::Position, http::Request, Load, WebView};
use winit::{ControlFlow, Event, EventsLoop, Window, WindowEvent};

fn main() {
    let mut events_loop = EventsLoop::new();
    let mut window = Window::new(&events_loop).unwrap();
    let size: (f64, f64) = window.get_inner_size().unwrap().into();

    let mut web_view = WebView::new(Position::new(0.0, 0.0), size.into(), |val| {
        eprintln!("invoke called with {}", val);
    });

    web_view.add_to(&mut window);
    web_view.load(Load::Request(
        Request::builder()
            .uri("https://www.google.com")
            .body(())
            .unwrap(),
    ));

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
