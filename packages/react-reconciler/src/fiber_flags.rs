use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone)]
    pub struct Flags: u8 {
        const NoFlags = 0b00000000;
        const Placement = 0b00000010;
        const Update = 0b00000100;
        const ChildDeletion = 0b00010000;
    }
}

pub fn get_mutation_mask() -> Flags {
    Flags::Placement | Flags::Update | Flags::ChildDeletion
}