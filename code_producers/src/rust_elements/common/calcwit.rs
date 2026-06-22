// Runtime witness calculator for circom-generated Rust code.
// Analogous to calcwit.cpp / Circom_CalcWit.

use malachite::integer::Integer;
use std::collections::BTreeMap;

pub type FrElement = Integer;

// ─── Component descriptor ─────────────────────────────────────────────────────

#[derive(Clone, Default)]
pub struct CircomComponent {
    pub template_id: usize,
    pub signal_start: usize,
    /// How many input signals are still unset (triggers run when 0)
    pub input_counter: usize,
    pub subcomponents: Vec<usize>,     // indices into component_memory
    pub subcomponents_parallel: Vec<bool>,
}

// ─── IO field descriptor (mirrors IODef / FieldData) ─────────────────────────

/// (offset, size, lengths, bus_id)
pub type IOField = (usize, usize, Vec<usize>, Option<usize>);

// ─── Main struct ──────────────────────────────────────────────────────────────

pub struct CircomCalcWit {
    pub signal_values: Vec<FrElement>,
    pub component_memory: Vec<CircomComponent>,
    pub circuit_constants: Vec<FrElement>,
    /// input_hash_map[i] = (hash, signal_index, size)
    pub input_hash_map: Vec<(u64, u64, u64)>,
    pub witness_to_signal: Vec<usize>,
    pub input_signal_start: usize,
    pub total_input_signals: usize,
    pub prime: Integer,
    pub io_map: BTreeMap<usize, Vec<IOField>>,
    pub bus_field_info: Vec<Vec<IOField>>,
    /// Fn pointer to the template dispatch table
    pub run_template: fn(usize, usize, &mut CircomCalcWit),
    /// Fn pointer to the top-level run (calls main_create)
    pub run_circuit: fn(&mut CircomCalcWit),
    pub remaining_input_signals: usize,
}

impl CircomCalcWit {
    pub fn new(
        n_signals: usize,
        n_components: usize,
        circuit_constants: Vec<FrElement>,
        input_hash_map: &[(u64, u64, u64)],
        witness_to_signal: &[usize],
        input_signal_start: usize,
        total_input_signals: usize,
        prime: Integer,
        io_map: BTreeMap<usize, Vec<IOField>>,
        bus_field_info: Vec<Vec<IOField>>,
        run_template: fn(usize, usize, &mut CircomCalcWit),
        run_circuit: fn(&mut CircomCalcWit),
    ) -> Self {
        CircomCalcWit {
            signal_values: (0..n_signals).map(|_| Integer::from(0u32)).collect(),
            component_memory: vec![CircomComponent::default(); n_components],
            circuit_constants,
            input_hash_map: input_hash_map.to_vec(),
            witness_to_signal: witness_to_signal.to_vec(),
            input_signal_start,
            total_input_signals,
            prime,
            io_map,
            bus_field_info,
            run_template,
            run_circuit,
            remaining_input_signals: total_input_signals,
        }
    }

    // ─── Signal access ────────────────────────────────────────────────────────

    /// Look up position in input_hash_map by FNV-1a hash of name
    pub fn find_hash(&self, name: &str) -> Option<(usize, usize)> {
        let h = fnv1a(name);
        let size = self.input_hash_map.len();
        if size == 0 {
            return None;
        }
        let mut pos = (h as usize) % size;
        loop {
            let (eh, esig, esz) = self.input_hash_map[pos];
            if eh == 0 && esig == 0 && esz == 0 {
                return None;
            }
            if eh == h {
                return Some((esig as usize, esz as usize));
            }
            pos = (pos + 1) % size;
        }
    }

    /// Set one element of an input signal array.
    /// Triggers witness computation when all inputs are filled.
    /// The hash map stores absolute global signal indices (dag_local_id values).
    pub fn set_input_signal(&mut self, name: &str, idx: usize, value: FrElement) {
        let (sig_start, _sig_size) = self
            .find_hash(name)
            .unwrap_or_else(|| panic!("Input signal '{}' not found", name));
        let abs_idx = sig_start + idx;
        assert!(
            abs_idx < self.signal_values.len(),
            "Signal index {} out of bounds",
            abs_idx
        );
        self.signal_values[abs_idx] = value;
        if self.remaining_input_signals == 0 {
            panic!("All inputs already set");
        }
        self.remaining_input_signals -= 1;
        if self.remaining_input_signals == 0 {
            // Avoid borrow: copy fn pointer, then call
            let f = self.run_circuit;
            f(self);
        }
    }

    /// Convenience: set a scalar input
    pub fn set_input(&mut self, name: &str, value: FrElement) {
        self.set_input_signal(name, 0, value);
    }

    // ─── Witness extraction ───────────────────────────────────────────────────

    pub fn get_witness_size(&self) -> usize {
        self.witness_to_signal.len()
    }

    pub fn get_witness(&self, idx: usize) -> &FrElement {
        let sig = self.witness_to_signal[idx];
        &self.signal_values[sig]
    }

    pub fn get_witness_vec(&self) -> Vec<FrElement> {
        (0..self.get_witness_size())
            .map(|i| self.get_witness(i).clone())
            .collect()
    }

    // ─── Component helpers ─────────────────────────────────────────────────────

    /// Register a component at component index, wiring it into the signal array
    pub fn create_component(
        &mut self,
        component_idx: usize,
        template_id: usize,
        signal_start: usize,
        input_count: usize,
        subcomponent_count: usize,
    ) {
        self.component_memory[component_idx] = CircomComponent {
            template_id,
            signal_start,
            input_counter: input_count,
            subcomponents: vec![0usize; subcomponent_count],
            subcomponents_parallel: vec![false; subcomponent_count],
        };
    }

    /// Called by generated _create code after setting up a component.
    /// Runs it immediately if it has no inputs.
    pub fn try_run_component(&mut self, component_idx: usize) {
        if self.component_memory[component_idx].input_counter == 0 {
            let tid = self.component_memory[component_idx].template_id;
            let f = self.run_template;
            f(tid, component_idx, self);
        }
    }

    /// Decrement the input counter of a component; run if ready.
    pub fn decrement_component_inputs(&mut self, component_idx: usize) {
        let cnt = self.component_memory[component_idx].input_counter;
        if cnt == 0 {
            panic!("Component {} received more inputs than expected", component_idx);
        }
        self.component_memory[component_idx].input_counter = cnt - 1;
        if cnt - 1 == 0 {
            let tid = self.component_memory[component_idx].template_id;
            let f = self.run_template;
            f(tid, component_idx, self);
        }
    }

    // ─── IO map helpers ───────────────────────────────────────────────────────

    pub fn get_io_field(
        &self,
        template_id: usize,
        signal_code: usize,
    ) -> &IOField {
        &self.io_map[&template_id][signal_code]
    }

    pub fn get_bus_field(&self, bus_id: usize, field_code: usize) -> &IOField {
        &self.bus_field_info[bus_id][field_code]
    }
}

// ─── FNV-1a hash (matches hasher() in components/mod.rs) ─────────────────────

pub fn fnv1a(s: &str) -> u64 {
    let mut hash: u64 = 0xCBF29CE484222325;
    for b in s.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001B3);
    }
    hash
}
