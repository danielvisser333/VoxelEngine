pub mod keyhandler;
use keyhandler::Keyhandler;
use crate::renderer::vulkan::Implementation;
use crate::renderer::camera::Camera;
use crate::renderer;
pub struct Application{
    pub renderer : Implementation,
    pub camera : Camera,
    pub keyhandler : Keyhandler,
}
impl Application{
    pub fn draw_frame(&mut self){
        self.renderer.draw_frame(&self.camera);
    }
    pub fn flush_and_refill_vertex_buffer(&mut self,vertices : Vec<renderer::vulkan::Vertex>,indices : Vec<u32>){
        self.renderer.flush_and_refill_vertex_buffer(vertices, indices)
    }
}