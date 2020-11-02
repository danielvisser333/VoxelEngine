use rand::SeedableRng;
use rand::Rng;
use rand::rngs::SmallRng;
pub struct SeededRng{
    rng : SmallRng,
}
impl SeededRng{
    pub fn new(seed : u64) -> Self{
        let rng = SmallRng::seed_from_u64(seed);
        return Self{
            rng
        }
    }
    pub fn get_float(&mut self) -> f32{
        return self.rng.gen::<f32>();
    }
    pub fn get_u64(&mut self) -> u64{
        return self.rng.gen::<u64>();
    }
    pub fn get_u64_ranged(&mut self , min : u64 , max : u64) -> u64{
        return self.rng.gen_range(min,max);
    }
}