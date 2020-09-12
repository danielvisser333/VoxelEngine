mod renderer;
mod game;
use renderer::vulkan::Vertex;
fn main(){
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&event_loop).expect("Failed to create window.");
    window.set_title("RendererTest");
    let mut renderer = renderer::vulkan::Implementation::new(true,&window);
    let camera = renderer::camera::Camera::new(renderer.swapchain.extent.width as f32,renderer.swapchain.extent.height as f32);
    let mut frame_counter = 0;
    let mut last_framerate_count = std::time::Instant::now();
    let start_time = std::time::Instant::now();
    let mut has_flushed = false;
    event_loop.run(move |event,_,control_flow|{
        *control_flow = winit::event_loop::ControlFlow::Poll;
        frame_counter+=1;
        match event{
            winit::event::Event::WindowEvent{
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = winit::event_loop::ControlFlow::Exit
            }
            winit::event::Event::MainEventsCleared => {
                if start_time.elapsed() >= std::time::Duration::from_secs(5) && !has_flushed{
                    has_flushed = true;
                    let vertices = vec!(
                        Vertex{color:[1.0,0.0,1.0],pos:[-0.9,-0.9]},
                        Vertex{color:[1.0,0.0,1.0],pos:[0.9,-0.9]},
                        Vertex{color:[1.0,0.0,0.0],pos:[0.9,0.9]},
                        Vertex{color:[0.0,1.0,0.0],pos:[-0.9,0.9]},
                    );
                    let indices = vec!(0,1,2,2,3,0);
                    renderer.flush_and_refill_vertex_buffer(vertices, indices);
                }
                if last_framerate_count.elapsed().as_secs()>=1{
                    println!("Framerate : {}",frame_counter);
                    frame_counter = 0;
                    last_framerate_count = std::time::Instant::now();
                }
                window.request_redraw();
            }
            winit::event::Event::RedrawRequested(_) => {
                
                renderer.draw_frame(&camera);
            }
            _=>{}
        }
    });
}
