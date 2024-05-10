use crate::fiber_lanes::Lane;

mod fiber_lanes;

fn main() {
    let mut a = Lane::NoLane | Lane::SyncLane;
    println!("{:?}", a);
    println!("{:?}", a == !Lane::NoLane)
}