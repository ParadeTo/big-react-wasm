use bitflags::bitflags;
use scheduler::{unstable_get_current_priority_level, Priority};

bitflags! {
    #[derive(Debug, Clone)]
    pub struct Lane: u32 {
        const NoLane =              0b0000000000000000000000000000000;
        const SyncLane =            0b0000000000000000000000000000001; // onClick
        const InputContinuousLane = 0b0000000000000000000000000000010; // Continuous Trigger, example: onScroll
        const DefaultLane =         0b0000000000000000000000000000100; // useEffect
        const IdleLane =            0b1000000000000000000000000000000;
    }
}

impl PartialEq for Lane {
    fn eq(&self, other: &Self) -> bool {
        self.bits() == other.bits()
    }
}

impl Eq for Lane {}

pub fn get_highest_priority(lanes: Lane) -> Lane {
    let lanes = lanes.bits();
    let highest_priority = lanes & (lanes.wrapping_neg());
    Lane::from_bits_truncate(highest_priority)
}

pub fn merge_lanes(lane_a: Lane, lane_b: Lane) -> Lane {
    lane_a | lane_b
}

pub fn is_subset_of_lanes(set: Lane, subset: Lane) -> bool {
    (set & subset.clone()) == subset
}

pub fn request_update_lane() -> Lane {
    let current_scheduler_priority_level = unstable_get_current_priority_level();
    let update_lane = scheduler_priority_to_lane(current_scheduler_priority_level);
    update_lane
}

pub fn scheduler_priority_to_lane(scheduler_priority: Priority) -> Lane {
    match scheduler_priority {
        Priority::ImmediatePriority => Lane::SyncLane,
        Priority::UserBlockingPriority => Lane::InputContinuousLane,
        Priority::NormalPriority => Lane::DefaultLane,
        _ => Lane::NoLane,
    }
}

pub fn lanes_to_scheduler_priority(lanes: Lane) -> Priority {
    let lane = get_highest_priority(lanes);
    if lane == Lane::SyncLane {
        return Priority::ImmediatePriority;
    } else if lane == Lane::InputContinuousLane {
        return Priority::UserBlockingPriority;
    } else if lane == Lane::DefaultLane {
        return Priority::NormalPriority;
    }
    Priority::IdlePriority
}
