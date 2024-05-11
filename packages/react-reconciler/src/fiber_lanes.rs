use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Lane: u8 {
        const NoLane = 0b0000000000000000000000000000000;
        const SyncLane = 0b0000000000000000000000000000001;
        // const AsyncLane = 0b0000000000000000000000000000010;
    }
}

pub fn get_highest_priority(lanes: Lane) -> Lane {
    let lanes = lanes.bits();
    let highest_priority = lanes & (lanes.wrapping_neg());
    Lane::from_bits_truncate(highest_priority)
}

pub fn merge_lanes(lane_a: Lane, lane_b: Lane) -> Lane {
    lane_a | lane_b
}

pub fn request_update_lane() -> Lane {
    Lane::SyncLane
}

