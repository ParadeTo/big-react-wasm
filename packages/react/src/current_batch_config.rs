pub struct ReactCurrentBatchConfig {
    pub transition: u8,
}

pub static mut REACT_CURRENT_BATCH_CONFIG: ReactCurrentBatchConfig =
    ReactCurrentBatchConfig { transition: 0 };
