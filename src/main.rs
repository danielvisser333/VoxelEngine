mod game;
mod renderer;

use game::seedrng;
use game::world::Chunk;
use game::GameState;

use renderer::vertex::Vertex;

fn main() {
    let mut rng = seedrng::SeededRng::new(123455);
    let game_state = GameState::new_temp();
    let chunks = vec!(
        Chunk::new_rand([0,0,0], &game_state, &mut rng),
        Chunk::new_rand([1,0,0], &game_state, &mut rng),
        Chunk::new_rand([0,1,0], &game_state, &mut rng),
        Chunk::new_rand([0,0,1], &game_state, &mut rng),
    );
    for chunk in chunks.iter(){
        chunk.print_chunk();
    }
}
