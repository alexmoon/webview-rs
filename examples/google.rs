use std::convert::Into;
use webview::{geometry::Position, http::Request, Load, WebView};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

fn main() {
    let events_loop = EventLoop::new();
    let mut window = Window::new(&events_loop).unwrap();
    let size: (f64, f64) = window
        .inner_size()
        .to_logical::<f64>(window.scale_factor())
        .into();

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

    println!("URI: {:?}", web_view.get_uri());

    events_loop.run(move |event, _, control_flow| {
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            println!("The close button was pressed; stopping");
            *control_flow = ControlFlow::Exit;
        }
    });
}
