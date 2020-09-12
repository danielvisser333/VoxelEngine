pub struct Camera{
    pub model_matrix : cgmath::Matrix4<f32>,
    pub view_matrix : cgmath::Matrix4<f32>,
    pub proj_matrix : cgmath::Matrix4<f32>,
}
impl Camera{
    pub fn new(width : f32 , height : f32) -> Self{
        let model_matrix = cgmath::Matrix4::from_axis_angle(cgmath::Vector3::new(0.0, 0.0, 1.0), cgmath::Rad(2.0));
        let view_matrix = cgmath::Matrix4::look_at(
            cgmath::Point3::new(2.0, 2.0, 2.0),
            cgmath::Point3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, 0.0, 1.0),
        );
        let proj_matrix = cgmath::perspective(cgmath::Rad(0.5) , width / height, 0.1, 10.0);
        return Self{model_matrix,view_matrix,proj_matrix};
    }
    pub fn move_camera(&mut self , delta_x : f32 , delta_y : f32 , delty_z : f32){
        
    }
}