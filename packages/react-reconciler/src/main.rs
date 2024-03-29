use react_reconciler::fiber::Flags;

fn main() {
    let a = Flags::NoFlags;
    let b = Flags::Placement | Flags::Update;
    println!("{:?}, {:?}", b.bits(), b.bits());
}