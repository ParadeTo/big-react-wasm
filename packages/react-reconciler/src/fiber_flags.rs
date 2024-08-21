use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone)]
    pub struct Flags: u16 {
        const NoFlags       = 0b00000000;
        const Placement     = 0b00000001;
        const Update        = 0b00000010;
        const ChildDeletion = 0b00000100;
        const PassiveEffect = 0b00001000;
        const Ref           = 0b00010000;
        const Visibility    = 0b00100000;
        const DidCapture    = 0b01000000;
        const ShouldCapture = 0b1000000000000;

        const LayoutMask    = 0b00010000; // Ref
        // HookEffectTags
        const HookHasEffect = 0b0001;
        const Passive = 0b0010; // useEffect
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

pub fn get_host_effect_mask() -> Flags {
    get_mutation_mask() | Flags::LayoutMask | get_passive_mask() | Flags::DidCapture
}
