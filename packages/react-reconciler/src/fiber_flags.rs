use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone)]
    pub struct Flags: u8 {
        const NoFlags = 0b00000000;
        const Placement = 0b00000010;
        const Update = 0b00000100;
        const ChildDeletion = 0b00010000;
        const PassiveEffect = 0b00100000;
        const Ref = 0b01000000;
        const LayoutMask = 0b01000000; // Ref
        // effect hook
        const HookHasEffect = 0b00100001;
        const Passive = 0b00000010;
    }
}

impl PartialEq for Flags {
    fn eq(&self, other: &Self) -> bool {
        self.bits() == other.bits()
    }
}

pub fn get_mutation_mask() -> Flags {
    Flags::Placement | Flags::Update | Flags::ChildDeletion
}

pub fn get_passive_mask() -> Flags {
    Flags::PassiveEffect | Flags::ChildDeletion
}
