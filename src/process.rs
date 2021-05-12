// by Flux-Audio, some rights reserved
// This code is licensed under MIT license (see LICENSE.md for details)

// third-party libraries
use vst::buffer::AudioBuffer;

// Flux-Audio libraries
use dsp_lab::traits::Process;

// internal dependencies
use super::Effect;

pub fn process_chunk(parent: &mut Effect, buffer: &mut AudioBuffer<f32>) {

    // === get parameters === parameter scaling ===
    // you only need to read parameters once per chunk, as that is as fast as the
    // host will update them.
    let some_parameter_raw = parent.params.params[0].get() as f64;
    // NOTE: get your parameters here

    // === process audio buffers ===

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
        let some_parameter = parent.param_filters[0].step(some_parameter_raw);
        // NOTE: filter your parameters here

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
}