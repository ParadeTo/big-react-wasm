use react_reconciler::fiber_flags::{Flags, get_mutation_mask};

fn main() {
    let a = Flags::NoFlags;
    let b = Flags::Placement | Flags::Update;
    let c = b.clone() & get_mutation_mask();
    println!("{:?}, {:?}", Flags::Placement, get_mutation_mask() - b.clone());
}