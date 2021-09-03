use std::sync::Mutex;

use crate::parameter_manager::{Parameter, ParameterManager};

// VST_LAB calls this to determine which parameters are displayed in the plugin
// window (no GUI) and which parameters the plugin editor can read and write to
// (with GUI)
pub fn create_parameters() -> ParameterManager {
    
    let parameters = vec![
        Parameter {
            display_name: Mutex::new("test_param".to_string()),
            ..Default::default()
        },
    ];
    ParameterManager::from_vec(parameters)
}