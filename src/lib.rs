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

// internal dependencies
mod process;        // contains the processing loop for the active input buffer
mod algorithms;     // contains support structs and functions for processing
mod parameter_manager;
mod voice_allocator;
// mod editor;      // contains the editor (GUI) logic NOTE: uncomment for GUI
// mod widgets;     // contains rendering of custom ui elements NOTE: uncomment for GUI


// === GLOBALS ===
// add global constants and statics here


// === PARAMETERS ===



// === PLUGIN ===
// here the plugin struct is defined. This holds together all the different
// components of the plugin, and runs the main processing loop (the contents of
// which are defined in a function of the `process.rs` sub-module)

// NOTE: you may want to rename this to `Generator` in case you are not making an
// effect
pub struct Effect {
    // Store a handle to the plugin's parameter object.
    parameter_manager: Arc<ParameterManager>,

    // store a handle to the GUI NOTE: uncomment for GUI
    // editor: Option<EffectEditor>,

    // host info
    sr: f64,            // sample rate
    scale: f64,         // scaling factor for sample rate independence
    dt: f64,            // duration of each sample

}

// this is where the state of the plugin is initialized
impl Default for Effect {
    fn default() -> Self {
        // TODO: call to user code to initialize parameter manager
        Self {
            parameter_manager: Arc::new(ParameterManager::default()).clone(),

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
        self.parameter_manager.set_sr(self.sr as f32);

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

            // === parameter filtering ===
            // while we read parameters once per chunk, we want to filter them at
            // the same rate as the audio, as we do this to remove audio artifacts.
            self.parameter_manager.step_filter();
            let some_parameter = self.parameter_manager.params[0].filtered.get() as f64;

            // === macro mappings ===
            // the ui might have less parameters than the underlying logic implies,
            // so there is a mapping between the UI macros and the audio processing
            // NOTE: mappings go here

            // === micro mappings ===
            // function and process parameter mappings from macros
            // NOTE: mappings go here

            // === main signal chain(s) ===
            // here is where the signal routed from input to output is processed,
            // with the parameters set up previously
            // NOTE: processing goes here. The example is a simple gain plugin
            let mut l = *left_in as f64;
            let mut r = *right_in as f64;
            l *= some_parameter;
            r *= some_parameter;
            *left_out = l as f32;
            *right_out = r as f32;

            // === feedback and aux signal chain(s) ===
            // any other signal chain that isn't routed to the output is processed
            // here, for example feedback paths.
            // NOTE: processing goes here


        }

        // === post-processing cleanup ===
        // NOTE: here you undo any pre-processing

        // restore denormal flush-to zero mode to host's preference
        unsafe { _MM_SET_FLUSH_ZERO_MODE(prev_ftz); }
    }
}


// this macro contains bindings to the VST framework. This is where all the
// magic happens
plugin_main!(Effect);