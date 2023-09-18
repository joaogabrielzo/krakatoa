use anyhow::Result;
use learn_vulkan::krakatoa::Krakatoa;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

fn main() -> Result<()> {
    /* Window */
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Learn Vulkan")
        .build(&event_loop)?;
    let krakatoa = Krakatoa::init(window)?;

    use winit::event::{Event, WindowEvent};
    event_loop.run(move |event, _, controlflow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *controlflow = winit::event_loop::ControlFlow::Exit;
        }
        Event::MainEventsCleared => {
            // doing the work here (later)
            krakatoa.window.request_redraw();
        }
        Event::RedrawRequested(_) => {
            //render here (later)
        }
        _ => {}
    });
}
