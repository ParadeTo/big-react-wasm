use bitflags::bitflags;

bitflags! {
    #[derive(Debug)]
    pub struct HookEffectTags: u8 {
        const HookHasEffect = 0b0001;
        const Passive = 0b0010; // useEffect
    }
}
