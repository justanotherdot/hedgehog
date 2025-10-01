use hedgehog::*;
use hedgehog_derive::Generate;

#[derive(Generate, Debug, Clone, PartialEq)]
struct Simple {
    value: i32,
}

fn main() {
    let gen = Simple::generate();
    let seed = Seed::random();
    let size = Size::new(10);
    let tree = gen.generate(size, seed);
    let simple = tree.outcome();
    println!("Generated: {simple:?}");
}
