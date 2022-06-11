use dbpnoise::*;
fn main() {

    let gen = gen_noise("ExtremelyLongVeryRandomHash!!",360, 16, 255, 0.1, 1.1);
    println!("{}", gen.len());
    println!("{}", gen[0].len());
}
