// by Flux-Audio, some rights reserved
// This code is licensed under MIT license (see LICENSE.md for details)

#![allow(non_snake_case)]

#[macro_use]
extern crate vst;

#[macro_use]
extern crate dsp_lab;

// third-party libraries
use vst::buffer::AudioBuffer;
// use vst::editor::Editor;     NOTE: uncomment for GUI
use vst::plugin::{Category, Info, Plugin, PluginParameters, CanDo};
use vst::util::AtomicFloat;
use vst::api::Supported;
use vst::api::Supported::*;
use vst::plugin::CanDo::*;

// Flux-Audio libraries
use dsp_lab::traits::{Source, Process};
use dsp_lab::core::lin_filter::LowPass1P;

// standard libraries
use std::sync::Arc;
use std::f64::consts;
use core::arch::x86_64::{
    _MM_FLUSH_ZERO_ON,
    _MM_SET_FLUSH_ZERO_MODE,
    _MM_GET_FLUSH_ZERO_MODE
};

// internal dependencies
mod process;        // contains the processing loop for the active input buffer
mod algorithms;     // contains support structs and functions for processing
// mod editor;      // contains the editor (GUI) logic NOTE: uncomment for GUI
// mod widgets;     // contains rendering of custom ui elements NOTE: uncomment for GUI


// === GLOBALS ===
// add global constants and statics here


// === PARAMETERS ===
// TODO: move the entire parameter logic into a separate file with a ParameterManager
// struct.
// parameters struct of vst plugin is defined here, with default values, getters
// and setters.

// Effect parameters are stored in a vector, indexed by the parameter id the host
// passes to the callback functions.
pub struct EffectParameters {
    params: Vec<AtomicFloat>,
}

// here is where initial values of parameters are set. These values show up when
// first instantiating the plugin.
impl Default for EffectParameters {
    fn default() -> Self {
        let mut ret = Self {
            params: Vec::new(),
        };

        // add a push for each parameter
        ret.params.push(AtomicFloat::new(0.5));

        return ret;
    }
}

// here is where the callback functions the host will call to get and set
// parameter values and parameter names.
impl PluginParameters for EffectParameters {

    // this bit is always the same
    fn get_parameter(&self, index: i32) -> f32 {
        match self.params.get(index as usize) {
            Some(p) => p.get(),
            None => 0.0,
        }
    }

    // this bit is always the same
    fn set_parameter(&self, index: i32, val: f32) {
        match self.params.get(index as usize) {
            Some(p) => p.set(val),
            None => (),
        }
    }

    // this returns the numeric readout of the parameter value. Format this to
    // to display ranges outside 0.0..1.0, adding units, ...
    fn get_parameter_text(&self, index: i32) -> String {

        // the parameter index corresponds to the order they were pushed into
        // the parameter vector.
        // NOTE: this will panic, if the specified parameter index is out of bounds
        match index {
            0 => format!("{:.2}", self.params.get(0).unwrap().get()),
            _ => "".to_string(),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {

        // the parameter index corresponds to the order they were pushed into
        // the parameter vector.
        match index {
            0 => "parameter name goes here",
            _ => "",
        }.to_string()
    }

    // TODO: missing methods for preset management
}


// === PLUGIN ===
// here the plugin struct is defined. This holds together all the different
// components of the plugin, and runs the main processing loop (the contents of
// which are defined in a function of the `process.rs` sub-module)

// NOTE: you may want to rename this to `Generator` in case you are not making an
// effect
pub struct Effect {
    // Store a handle to the plugin's parameter object.
    params: Arc<EffectParameters>,

    // store a handle to the GUI NOTE: uncomment for GUI
    // editor: Option<EffectEditor>,

    // host info
    sr: f64,            // sample rate
    scale: f64,         // scaling factor for sample rate independence
    dt: f64,            // duration of each sample

    // parameter filters
    // NOTE: these are used to de-click parameter tweaking during playback
    param_filters: Vec<LowPass1P>,

    // NOTE: any stateful processing requires the state to be declared here, i.e.
    // any dsp_lab process or generator.

    // ... your stuff goes here ...

}

// this is where the state of the plugin is initialized
impl Default for Effect {
    fn default() -> Self {
        let params = Arc::new(EffectParameters::default());
        let param_filters = vec![LowPass1P::new()];     // TODO: change dsplab
        
        Self {
            params: params.clone(),

            // host info. these should not be left to their default value
            // they are changed at runtime with the set_sample_rate() callback
            sr: 44100.0,
            scale: 1.0,
            dt: 1.0 / 44100.0,

            // parameter filters
            param_filters: param_filters,

            // ... your stuff goes here ...
        }
    }
}

// All plugins using `vst` also need to implement the `Plugin` trait.  Here, we
// define functions that give necessary info to our host.
impl Plugin for Effect {

    // used by host to learn about the plugin's characteristics upon initialization
    fn get_info(&self) -> Info {

        // NOTE: remember to change these!
        Info {
            name: "TEMPLATE_PLUGIN".to_string(),
            vendor: "Flux-Audio".to_string(),
            unique_id: 0,       // NOTE: by own convention, this should be the Adler-32
                                // hash of name + version, i.e. `TEMPLATE_PLUGIN v0.1.x`
            version: 00_01_00,  // from semantic versioning number: xx.xx.xx
            inputs: 2,          // NOTE: zero if this is a generator
            outputs: 2,
            parameters: 1,
            category: Category::Effect, // NOTE: use `Category::Generator` if it's a generator
            initial_delay: 0,   // minimum processing latency, that the host can use for
                                // for latency compensation
            ..Default::default()
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.sr = rate as f64;
        self.scale = 44100.0 / rate as f64;
        self.dt = 1.0 / rate as f64;
    }

    // NOTE: use this to run code that needs to be run only once after the plugin is
    // instantiated
    fn init(&mut self) {

        self.param_filters[0].set_sr(self.sr);      // TODO: implement this in dsplab
        self.param_filters[0].set_cutoff(5.0);

        // ... your stuff goes here ...
    }

    // NOTE: host uses this to find out what features this plugin supports, i.e. if it
    // can take MIDI inputs
    fn can_do(&self, can_do: CanDo) -> Supported {
        match can_do {
            SendEvents | ReceiveEvents => Yes,
            _ => No,
        }
    }
    
    // Return the parameter object. This method can be omitted if the
    // plugin has no parameters.
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }

    // TODO: not sure when exactly this is called
    fn resume(&mut self) {}

    // TODO: not sure when exactly this is called
    fn suspend(&mut self) {}

    // Here is where the bulk of our audio processing code goes.
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        
        // === pre-processing setup ===
        // NOTE: here is any environment alterning pre-operation that needs
        // to be executed once before processing, but which needs to be
        // undone before returning control to the host

        // disable denormal floats in CPU for performance improvements
        let prev_ftz = unsafe { _MM_GET_FLUSH_ZERO_MODE() };
        unsafe { _MM_SET_FLUSH_ZERO_MODE(_MM_FLUSH_ZERO_ON); }

        // === process audio buffer ===

        process::process_chunk(self, buffer);

        // === post-processing cleanup ===
        // NOTE: here you undo any pre-processing

        // restore denormal flush-to zero mode to host's preference
        unsafe { _MM_SET_FLUSH_ZERO_MODE(prev_ftz); }
    }
}


// this macro contains bindings to the VST framework. This is where all the
// magic happens
plugin_main!(Effect);