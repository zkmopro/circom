pub mod rust_code_generator;

pub use crate::components::*;

pub type RustInstruction = String;

pub struct RustProducer {
    pub main_header: String,
    pub main_is_parallel: bool,
    pub has_parallelism: bool,
    pub number_of_main_outputs: usize,
    pub main_signal_offset: usize,
    pub number_of_main_inputs: usize,
    pub signals_in_witness: usize,
    pub total_number_of_signals: usize,
    pub number_of_components: usize,
    pub size_of_component_tree: usize,
    pub prime: String,
    pub prime_str: String,
    pub main_input_list: InputList,
    pub witness_to_signal_list: SignalList,
    pub io_map: TemplateInstanceIOMap,
    pub template_instance_list: TemplateListInfo,
    pub message_list: MessageList,
    pub string_table: Vec<String>,
    pub field_tracking: Vec<String>,
    pub major_version: usize,
    pub minor_version: usize,
    pub patch_version: usize,
    pub num_of_bus_instances: usize,
    pub busid_field_info: FieldMap,
}

impl Default for RustProducer {
    fn default() -> Self {
        RustProducer {
            main_header: "Main_0".to_string(),
            main_is_parallel: false,
            has_parallelism: false,
            main_signal_offset: 1,
            prime: "21888242871839275222246405745257275088548364400416034343698204186575808495617"
                .to_string(),
            prime_str: "bn128".to_string(),
            number_of_main_outputs: 1,
            number_of_main_inputs: 2,
            main_input_list: vec![],
            signals_in_witness: 20,
            witness_to_signal_list: vec![0, 1],
            message_list: vec![],
            string_table: vec![],
            field_tracking: vec![],
            total_number_of_signals: 80,
            number_of_components: 4,
            size_of_component_tree: 3,
            io_map: TemplateInstanceIOMap::new(),
            template_instance_list: Vec::new(),
            major_version: 0,
            minor_version: 0,
            patch_version: 0,
            num_of_bus_instances: 0,
            busid_field_info: Vec::new(),
        }
    }
}

impl RustProducer {
    pub fn get_prime(&self) -> &str {
        &self.prime
    }
    pub fn get_prime_str(&self) -> &str {
        &self.prime_str
    }
    pub fn get_main_header(&self) -> &str {
        &self.main_header
    }
    pub fn get_main_is_parallel(&self) -> bool {
        self.main_is_parallel
    }
    pub fn get_number_of_main_inputs(&self) -> usize {
        self.number_of_main_inputs
    }
    pub fn get_number_of_main_outputs(&self) -> usize {
        self.number_of_main_outputs + 1
    }
    pub fn get_main_signal_offset(&self) -> usize {
        self.main_signal_offset
    }
    pub fn get_main_input_list(&self) -> &InputList {
        &self.main_input_list
    }
    pub fn get_input_hash_map_entry_size(&self) -> usize {
        std::cmp::max(
            usize::pow(2, (self.main_input_list.len() as f32).log2().ceil() as u32),
            256,
        )
    }
    pub fn get_number_of_witness(&self) -> usize {
        self.signals_in_witness
    }
    pub fn get_witness_to_signal_list(&self) -> &SignalList {
        &self.witness_to_signal_list
    }
    pub fn get_total_number_of_signals(&self) -> usize {
        self.total_number_of_signals
    }
    pub fn get_number_of_components(&self) -> usize {
        self.number_of_components
    }
    pub fn get_io_map(&self) -> &TemplateInstanceIOMap {
        &self.io_map
    }
    pub fn get_template_instance_list(&self) -> &TemplateListInfo {
        &self.template_instance_list
    }
    pub fn get_number_of_template_instances(&self) -> usize {
        self.template_instance_list.len()
    }
    pub fn get_message_list(&self) -> &MessageList {
        &self.message_list
    }
    pub fn get_field_constant_list(&self) -> &Vec<String> {
        &self.field_tracking
    }
    pub fn get_number_of_bus_instances(&self) -> usize {
        self.num_of_bus_instances
    }
    pub fn get_busid_field_info(&self) -> &FieldMap {
        &self.busid_field_info
    }
    pub fn get_string_table(&self) -> &Vec<String> {
        &self.string_table
    }
    pub fn set_string_table(&mut self, table: Vec<String>) {
        self.string_table = table;
    }
}
