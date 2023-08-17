pub struct ADIn(libjulius_sys::ADIn);

impl ADIn {
    pub(crate) fn samp_num(&self) -> i32 {
        self.0.bpmax - self.0.bp
    }
    pub(crate) fn ad_read_hack_prepare(&mut self) {
        self.0.ad_standby = None;
        self.0.ad_begin = None;
        self.0.ad_end = None;
        self.0.ad_resume = None;
        self.0.ad_pause = None;
        self.0.ad_terminate = None;
        self.0.ad_read = Some(Self::dummy_ad_read);
        self.0.ad_input_name = None;
        self.0.silence_cut_default = 0;
        self.0.enable_thread = 0;
        self.0.level_coef = 1.0;
        self.0.down_sample = 0;
        self.0.strip_flag = 0;
        self.0.need_zmean = 0;
    }
    pub(crate) fn ad_read_hack_callback<T: AsRef<[i16]>>(&mut self, data: Option<T>) {
        if let Some(data) = data {
            let cnt = data.as_ref().len();
            assert!(cnt as i32 <= self.samp_num());
            // Subtract 1 to compensate for dummy cnt returned from dummy_ad_read
            self.0.total_captured_len = self.0.total_captured_len + cnt as u32 - 1;
            self.0.current_len = self.0.bp + cnt as i32 - 1;
            // Copy data
            unsafe {
                self.0
                    .buffer
                    .offset(self.0.bp as isize)
                    .copy_from(data.as_ref().as_ptr(), cnt);
            }
        } else {
            self.0.input_side_segment = 0;
            self.0.end_of_stream = 1;
        }
    }
    extern "C" fn dummy_ad_read(_buf: *mut libjulius_sys::SP16, _sampnum: i32) -> i32 {
        1
    }
}
