pub struct ReactCurrentBatchConfig {
    pub transition: u32,
}

pub static mut REACT_CURRENT_BATCH_CONFIG: ReactCurrentBatchConfig =
    ReactCurrentBatchConfig { transition: 0 };
