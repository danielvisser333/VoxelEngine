use crate::game::seedrng::SeededRng;
use crate::game::GameState;
use crate::renderer::vertex::Vertex;
pub struct Chunk{
    position : [u32;3],
    blocks : [[[u32;32];32];32],
}
impl Chunk{
    //Purely a debug function, should not be used in game
    pub fn new_rand(position : [u32;3], game_state : &GameState , rng : &mut SeededRng) -> Self{
        let mut blocks = [[[0;32];32];32];
        for x in 0..32{
            for y in 0..32{
                for z in 0..32{
                    blocks[x][y][z] = Chunk::rand_block(game_state, rng);
                }
            }
        }
        return Self{
            position,
            blocks,
        };
    }
    fn rand_block(game_state : &GameState , rng : &mut SeededRng)->u32{
        let id =  rng.get_u64_ranged(0, game_state.blockids.len() as u64);
        return id as u32;
    }
    pub fn print_chunk(&self){
        println!("Chunk:");
        for &two_dimensional_chunk in self.blocks.iter(){
            for &row in two_dimensional_chunk.iter(){
                for &block in row.iter(){
                    print!("{}",if block==0{"_".to_string()}else{block.to_string()});
                }
                println!(",Endrow,");
            }
            println!(",Endlayer,");
        }
    }
    pub fn to_vertices(&self) -> Vec<Vertex>{
        
        let mut vertices = vec!();
        //TODO : Convert all blocks into vertices, and then later decide which will be rendered.
        return vertices;
    }
}