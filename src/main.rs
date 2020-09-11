mod renderer;
use renderer::vulkan::Vertex;
fn main(){
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&event_loop).expect("Failed to create window.");
    window.set_title("RendererTest");
    let mut renderer = renderer::vulkan::Implementation::new(true,&window);
    let start_time = std::time::Instant::now();
    let mut has_flushed = false;
    event_loop.run(move |event,_,control_flow|{
        *control_flow = winit::event_loop::ControlFlow::Poll;
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
                window.request_redraw();
            }
            winit::event::Event::RedrawRequested(_) => {
                let model_matrix = cgmath::Matrix4::from_axis_angle(cgmath::Vector3::new(0.0, 0.0, 1.0), cgmath::Rad(0.5) * start_time.elapsed().as_secs_f32());
                let view_matrix = cgmath::Matrix4::look_at(
                    cgmath::Point3::new(2.0, 2.0, 2.0),
                    cgmath::Point3::new(0.0, 0.0, 0.0),
                    cgmath::Vector3::new(0.0, 0.0, 1.0),
                );
                let proj_matrix = cgmath::perspective(
                    cgmath::Rad(0.5),
                    renderer.swapchain.extent.width as f32
                        / renderer.swapchain.extent.height as f32,
                    0.1,
                    10.0,
                );
                renderer.draw_frame(&model_matrix,&view_matrix,&proj_matrix);
            }
            _=>{}
        }
    });
}
