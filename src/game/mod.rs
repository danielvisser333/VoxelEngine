pub mod seedrng;
pub mod world;

pub struct GameState{
    blockids : Vec<String>,
}
impl GameState{
    pub fn new_temp() -> Self{
        return Self{
            blockids : vec!("Air".to_string(),"".to_string(),"".to_string(),"".to_string()),
        }
    }
}