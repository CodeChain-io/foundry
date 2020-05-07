pub trait FeeManager {
    fn accumulate_block_fee(&self, total_additional_fee: u64, total_min_fee: u64);
}

#[allow(dead_code)]
pub fn fee_manager() -> Box<dyn FeeManager> {
    unimplemented!()
}
