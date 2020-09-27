mod renderer;
mod application;
use application::Application;
use renderer::vulkan::Vertex;
fn main(){
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&event_loop).expect("Failed to create window.");
    window.set_title("RendererTest");
    let mut frame_counter = 0;
    let mut last_framerate_count = std::time::Instant::now();
    let start_time = std::time::Instant::now();
    let mut has_flushed = false;
    let size = window.inner_size();
    let mut application = Application{renderer:renderer::vulkan::Implementation::new(false,&window),camera : renderer::camera::Camera::new(size.width as f32, size.height as f32),keyhandler : application::keyhandler::Keyhandler::new()};
    event_loop.run(move |event,_,control_flow|{
        *control_flow = winit::event_loop::ControlFlow::Poll;
        frame_counter+=1;
        match event{
            winit::event::Event::WindowEvent{event,..} => match event{
                winit::event::WindowEvent::CloseRequested =>{*control_flow = winit::event_loop::ControlFlow::Exit},
                winit::event::WindowEvent::KeyboardInput{..} => {},
                _ => {}
            }
            winit::event::Event::MainEventsCleared => {
                if start_time.elapsed() >= std::time::Duration::from_secs(2) && !has_flushed{
                    has_flushed = true;
                    let vertices = vec!(
                        Vertex{color:[1.0,0.0,1.0],pos:[-0.9,-0.9]},
                        Vertex{color:[1.0,0.0,1.0],pos:[0.9,-0.9]},
                        Vertex{color:[1.0,0.0,0.0],pos:[0.9,0.9]},
                        Vertex{color:[0.0,1.0,0.0],pos:[-0.9,0.9]},
                        Vertex{color:[0.5,0.5,1.0],pos:[-0.8,-0.8]},
                        Vertex{color:[0.7,0.0,1.0],pos:[0.8,-0.8]},
                        Vertex{color:[1.0,0.6,0.0],pos:[0.8,0.8]},
                        Vertex{color:[0.0,0.0,1.0],pos:[-0.8,0.8]},
                    );
                    let indices = vec!(0,1,2,2,3,0,5,6,7,7,8,5);
                    application.flush_and_refill_vertex_buffer(vertices, indices);
                }
                if last_framerate_count.elapsed().as_secs()>=1{
                    println!("Framerate : {}",frame_counter);
                    frame_counter = 0;
                    last_framerate_count = std::time::Instant::now();
                }
                window.request_redraw();
            }
            winit::event::Event::RedrawRequested(_) => {
                
                application.draw_frame();
            }
            _=>{}
        }
    });
}
