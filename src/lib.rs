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
use std::sync::{Arc, Mutex};
use std::f64::consts;
use std::collections::HashMap;
use core::arch::x86_64::{
    _MM_FLUSH_ZERO_ON,
    _MM_SET_FLUSH_ZERO_MODE,
    _MM_GET_FLUSH_ZERO_MODE
};

use crate::parameter_manager::{Parameter, ParameterManager};
use crate::user_code::parameters::create_parameters;
use crate::user_code::process_effects::EffectProcessor;


// internal dependencies
mod algorithms;     // contains support structs and functions for processing
mod parameter_manager;
mod voice_allocator;
mod user_code;
// mod editor;      // contains the editor (GUI) logic NOTE: uncomment for GUI
// mod widgets;     // contains rendering of custom ui elements NOTE: uncomment for GUI

pub struct VstLabPlugin {
    // Store a handle to the plugin's parameter object.
    parameter_manager: Arc<ParameterManager>,

    // This is where user code is located
    processor: EffectProcessor,

    // store a handle to the GUI NOTE: uncomment for GUI
    // editor: Option<EffectEditor>,

    // host info
    sr: f64,            // sample rate
    scale: f64,         // scaling factor for sample rate independence
    dt: f64,            // duration of each sample

}

// this is where the state of the plugin is initialized
impl Default for VstLabPlugin {
    fn default() -> Self {
        VstLabPlugin {
            parameter_manager: Arc::new(create_parameters()),
            processor: EffectProcessor::new(),

            // host info. these should not be left to their default value
            // they are changed at runtime with the set_sample_rate() callback
            sr: 44100.0,
            scale: 1.0,
            dt: 1.0 / 44100.0,
        }
    }
}

// All plugins using `vst` also need to implement the `Plugin` trait.  Here, we
// define functions that give necessary info to our host.
impl Plugin for VstLabPlugin {

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
            parameters: self.parameter_manager.params.len() as i32,
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

        self.processor.set_sr(rate as f64);
    }


    fn init(&mut self) { self.parameter_manager.set_sr(self.sr as f32); }

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
        self.parameter_manager.clone() as Arc<dyn PluginParameters>
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

        let (inputs, outputs) = buffer.split();
    
        // iterate over input and output buffers as references
        let (l, r) = inputs.split_at(1);
        let stereo_in = l[0].iter().zip(r[0].iter());
        let (mut l, mut r) = outputs.split_at_mut(1);
        let stereo_out = l[0].iter_mut().zip(r[0].iter_mut());
        for ((left_in, right_in), (left_out, right_out)) in stereo_in.zip(stereo_out) {

            self.parameter_manager.step_filter();

            let l = *left_in as f64;
            let r = *right_in as f64;
            let res = self.processor.process_effects(self.parameter_manager.clone(), l, r);
            *left_out = res.0 as f32;
            *right_out = res.1 as f32;
            
        }

        // restore denormal flush-to zero mode to host's preference
        unsafe { _MM_SET_FLUSH_ZERO_MODE(prev_ftz); }
    }
}

// this macro contains bindings to the VST framework. This is where all the
// magic happens
plugin_main!(VstLabPlugin);