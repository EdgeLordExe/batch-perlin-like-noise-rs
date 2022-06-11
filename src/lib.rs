use std::{ops::Sub, f32::consts::PI, sync::Mutex};

use lerp::Lerp;
use nalgebra::{Vector2, SimdValue};
use rand::{prelude::ThreadRng, Rng};
use rand_pcg::Pcg32;
use rand_seeder::Seeder;
use rayon::iter::{IntoParallelIterator, ParallelIterator, IndexedParallelIterator, IntoParallelRefMutIterator, IntoParallelRefIterator};

#[derive(Debug,Default,Clone)]
struct Stamp{
    data: Vec<Vec<f32>>
}

impl Stamp{
    fn new(vector: &Vector2<f32>,size: usize) -> Self{

        let mut v : Vec<Vec<f32>>= Vec::new();
        for x in 0..size{
            v.push(Vec::new());
            for _ in 0..size{
                v[x].push(0.0f32);
            }
        }

        let fsize = size as f32;
        for x in 0..size {
            let fx = x as f32;
            for y in 0..size {
                let fy = y as f32;
                v[x][y] = Vector2::new( fx - (fsize/2.0)+ 0.5 ,fy - (fsize/2.0)+ 0.5).scale(1.0/( (fsize/2.0)*2.0f32.sqrt())  ).dot(&vector);
            }
        }

        Stamp{
            data: v ,
        }
    }
}
//potentially replace this? i dunno
fn smoothstep(x: f32) -> f32{
    ((8.0 * x -3.0).tanh()+1.0) /2.0
}

fn gen_stamps(accuracy: u32,size: usize) -> Vec<Stamp>{
    let step : f32 = 2.0*PI / accuracy as f32;
    (0..accuracy).into_par_iter().map(|i| {
        Stamp::new(&Vector2::new((i as f32*step).cos(), (i as f32*step).sin()), size)
    }).collect()
}
/// Seed - the seed to use for the random number generator.
/// 
/// accuracy - Dictates how many stamps we create for generation purposes.
/// 
/// stamp_size - Size of stamp, roughly equivalent to frequency of standard perlin noise
/// 
/// world_size - Size of the plane
/// 
/// lower_range - Lower range of noise, if the value in a cell is greater than lower_range and less than upper_range, it will be marked as true, otherwise false
/// 
/// upper_range - Lower range of noise, if the value in a cell is greater than lower_range and less than upper_range, it will be marked as true, otherwise false
///
/// Alright so here is the explanation of the algorithm, perlin noise is created by first creating a grid array, each cell in the grid
/// is assigned a vector pointing in a random direction. that vector creates a gradient, pointing to 1 and having 0 in the other direction with smoothly interpolated values between i.e a gradient.
/// then we create another grid that we overlay on top of the original cell grid, we poll values at the location of every cell and we average it with nearby cells. thats how we get the classic noise.
///
/// This is not exactly perlin noise, but rather a smart reimplementation of it. Since we know that we are working on a grid we dont need to generate all the data necessary.
/// By creating N amount of "stamps" that we can "press" onto the returning grid we can skip the overhead of calculating the dot product a lot of times, we do it a single time and we just reuse the results.
/// we will call this N variable "accuracy"
/// as N approaches infinity we will approach the original implementation of perlin noise, but I wager accuracy of 8 or 16 should be enough to make a nice looking perlin-ish noise.
/// We will create N stamps with each stamp having a precalculated vector pointing in a direction (2 * PI/N) * i , meaning each vector will be exactly 2 * PI/N away from eachother.
/// Also another note, we do not use the original smoothstep function, i replaced it with a sigmoid tanh(x) as the big benefit of it is that it doesn't require clamping, as the codomain of tanh() is (-1;1)
pub fn gen_noise(seed: &str,accuracy: u32, stamp_size: usize, world_size: usize, lower_range: f32, upper_range: f32) -> Vec<Vec<bool>>{
    let real_stamp_size = stamp_size * 2;
    let mut rng : Pcg32 = Seeder::from(seed).make_rng();
    let stamps = gen_stamps(accuracy, real_stamp_size);
    //quite funky i know, but we need the world size to actually be a multiple of the real_stamp_size /shrug
    let real_world_size = (((world_size)as f32/stamp_size as f32).ceil() * stamp_size as f32 + real_stamp_size as f32) as usize;
    let mut stamp_vec : Vec<Vec<usize>> = Vec::new();
    for x in 0..real_world_size/stamp_size{
        stamp_vec.push(Vec::new());
        for _ in 0..real_world_size/stamp_size{
            stamp_vec[x].push(rng.gen::<usize>() % stamps.len());
        }
    }

    let mut res: Vec<Vec<bool>> = (real_stamp_size..real_world_size).into_par_iter().map(|x|{
        (real_stamp_size..real_world_size).into_iter().map(|y|{
            let xdiv = x/stamp_size;
            let ydiv = y/stamp_size;

            if xdiv < 1 || xdiv > stamp_vec.len() || ydiv < 1 || ydiv > stamp_vec.len() {
                return false;
            }

            let x1 = stamps[stamp_vec[xdiv][ydiv]].data[x - (xdiv * stamp_size)][y - (ydiv * stamp_size)];
            let x2 = stamps[stamp_vec[xdiv-1][ydiv]].data[x - ((xdiv-1) * stamp_size)][y - (ydiv * stamp_size)];
            let x3 = stamps[stamp_vec[xdiv][ydiv-1]].data[x - (xdiv * stamp_size)][y - ((ydiv-1) * stamp_size)];
            let x4 = stamps[stamp_vec[xdiv-1][ydiv-1]].data[x - ((xdiv-1) * stamp_size)][y - ((ydiv-1) * stamp_size)];
            let unit_coordinates = (smoothstep((x - (xdiv * stamp_size)) as f32 / stamp_size as f32 ),smoothstep((y - (ydiv * stamp_size)) as f32 / stamp_size as f32));
            let result = x4.lerp(x3, unit_coordinates.0).lerp(x2.lerp(x1, unit_coordinates.0), unit_coordinates.1);

            result >= lower_range && result < upper_range
        }).collect()
    }).collect();
    cut_noise_to_dimensions(&mut res, world_size);
    res
}

fn cut_noise_to_dimensions(vec: &mut Vec<Vec<bool>>, size: usize){
    vec.truncate(size);
    vec.par_iter_mut().for_each(|nested_vec|{
        nested_vec.truncate(size);
    });
}


//Prints the noise with X representing true, and a space representing false
pub fn print_noise(vec: &Vec<Vec<bool>>){
    for x in 0..vec.len(){
        for y in 0..vec[x].len(){
            if vec[x][y] {
                print!("X");
            } else {
                print!(" ");
            }
        }
        print!("\n");
    }
}
