use crate::parameter_manager::ParameterManager;
use dsp_lab::utils::math::{x_fade};
use dsp_lab::core::lin_filter::{DcBlock, SvfLowPass};
use dsp_lab::traits::Process;
use std::sync::Arc;
use std::collections::VecDeque;

pub struct EffectProcessor {
    // NOTE: effect state goes here
}

const RESOLUTION: usize = 512;

impl EffectProcessor {
    pub fn new() -> Self { 
        // NOTE: effect state initialization must happen here
        return EffectProcessor {}
    }

    pub fn set_sr(&mut self, sr: f64) {
        // NOTE: this is called by the framework to set the sample rate
    }

    pub fn process_effects(&mut self, param_mngr: Arc<ParameterManager>, l: f64, r: f64) -> (f64, f64) {
        let test_param = param_mngr.params[0].filtered.get() as f64;

        // NOTE: process L and R channels and return L and R 
        return (l, r);
    }
}

