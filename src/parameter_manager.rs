// by Flux-Audio, some rights reserved
// This code is licensed under MIT license (see LICENSE.md for details)

use vst::plugin::PluginParameters;
use vst::util::AtomicFloat;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Encapsulate parameter raw value, filtered value and formatting options.
/// 
/// `Parameter` Encapsulates the raw parameter value and the filtered value
/// the difference is that the raw value is set at a slower rate than the filtered
/// one by the DAW, the larger the buffer size, the lower the rate.
/// The value is upsampled by low-pass filtering, this is done in the process
/// loop. This avoids "zipper" noise when modulating parameter during playback.
pub struct Parameter {
    pub raw:        AtomicFloat,
    pub filtered:   AtomicFloat,
    pub display_name:   Mutex<String>,
    pub value_format:   Box<dyn Fn(f32) -> String + Sync + Send>,
    pub filter_cut: AtomicFloat,
}

impl Default for Parameter {
    fn default() -> Self {
        Self {
            raw:          AtomicFloat::new(0.0),
            filtered:     AtomicFloat::new(0.0),
            display_name: Mutex::new("".to_string()),
            value_format: Box::new(|x| format!("{:.2}", x)),
            filter_cut:   AtomicFloat::new(5.0),
        }
    }
}

impl Parameter {
    /// New parameter, specifying approximate decay time for paramether smoothing
    pub fn new(
        display_name: String, 
        value_format: Box<dyn Fn(f32) -> String + Sync + Send>, 
        decay_time: f32
    ) -> Self {
        Self {
            display_name: Mutex::new(display_name),
            value_format: value_format,
            filter_cut:   AtomicFloat::new(1.0 / decay_time),
            ..Default::default()
        }
    }
}

/// Stores all parameters, manages thread safe access to parameters, performs
/// smoothing of parameters.
/// 
/// Access parameters directly by subscripting the `params` vector. If their
/// index cannot easily be determined, use `get_param_by_name()` instead.
pub struct ParameterManager {
    pub params: Vec<Parameter>,
    params_indexes: HashMap<String, usize>,
    sr: AtomicFloat,
}

/// Default initializer creates a single unnamed parameter
impl Default for ParameterManager {
    fn default() -> Self {
        let mut ret = ParameterManager {
            params: Vec::new(),
            params_indexes: HashMap::new(),
            sr: AtomicFloat::new(44100.0),
        };

        ret.params.push(Parameter {
            ..Default::default()
        });
        ret.params_indexes.insert("".to_string(), 0);

        return ret;
    }
}

impl ParameterManager {

    /// Constructs a `ParameterManager` from a vector of `Parameter` objects.
    /// 
    /// Sample rate is set to its default value, if it needs to be changed, use
    /// `set_sr()`.
    pub fn from_vec(params: Vec<Parameter>) -> Self {
        let mut ret = ParameterManager {
            params: Vec::new(),
            params_indexes: HashMap::new(),
            sr: AtomicFloat::new(44100.0),
        };

        let mut idx: usize = 0;
        for p in params {
            ret.params_indexes.insert(p.display_name.lock().unwrap().clone(), idx);
            ret.params.push(p);
            idx += 1;
        }

        return ret;
    }

    /// Steps parameter smoothing, should be called once per processing loop
    /// to resample parameters to host sample rate.
    pub fn step_filter (&self) {
        for p in self.params.iter() {
            let alpha = 1.0 - (-std::f32::consts::TAU * p.filter_cut.get().clamp(0.0, self.sr.get()) / self.sr.get()).exp();
            p.filtered.set(p.filtered.get() + alpha * (p.raw.get() - p.filtered.get()));
        }
    }

    /// Set sample rate, should be called whenever the host changes its sampel rate
    /// i.e. in `init()` and `resume()`.
    pub fn set_sr(&self, sr: f32) { self.sr.set(sr) }

    /// You can index parameter objects by their name (not necessarily the same
    /// as their `display_name`).
    /// 
    /// The name of a parameter is the same as its `display_name` only if the
    /// `ParameterManager` has been initialized using `from_vec()`. Otherwise
    /// the parameter names can be set manually differently than their `display_name`.
    /// 
    /// It is suggested that the parameter is accessed directly through the internal
    /// vector it is stored in, unless its index cannot easily be determined.
    ///
    /// # Caveats
    /// The method returns an Option, which is `None` if the provided string did
    /// not match any of the parameter names.
    pub fn get_param_by_name(&self, name: String) -> Option<&Parameter> {
        match self.params_indexes.get(&name) {
            Some(idx) => Option::from(&self.params[*idx]),
            _ => None
        }
    }
}

// VST bindings need this
// NOTE: should not be changed! (Unless you know what you are doing)
impl PluginParameters for ParameterManager {

    fn get_parameter(&self, index: i32) -> f32 {
        match self.params.get(index as usize) {
            Some(p) => p.raw.get(),
            None => 0.0,
        }
    }

    fn set_parameter(&self, index: i32, val: f32) {
        match self.params.get(index as usize) {
            Some(p) => p.raw.set(val),
            None => (),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match self.params.get(index as usize) {
            Some(p) => (p.value_format)(p.raw.get()),
            _ => "".to_string(),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match self.params.get(index as usize) {

            // TODO: handle when mutex panics
            Some(p) => p.display_name.lock().unwrap().clone(),
            _ => "".to_string(),
        }
    }

    // TODO: missing methods for preset management
}