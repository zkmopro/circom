use super::*;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

// ─── Variable / storage names ────────────────────────────────────────────────

pub const L_LVAR: &str = "lvar";
pub const L_EXPAUX: &str = "expaux";
pub const L_LVARCALL: &str = "lvarcall";
pub const MY_SIGNAL_START: &str = "my_signal_start";
pub const MY_SUBCOMPONENTS: &str = "my_subcomponents";
pub const MY_SUBCOMPONENTS_PARALLEL: &str = "my_subcomponents_parallel";
pub const MY_ID: &str = "my_id";
pub const MY_FATHER: &str = "my_father";
pub const MY_TEMPLATE_NAME: &str = "my_template_name";
pub const CTX: &str = "ctx";
pub const DESTINATION: &str = "destination";
pub const DESTINATION_SIZE: &str = "destination_size";
pub const CMP_INDEX_REF: &str = "cmp_index_ref";
pub const CMP_INDEX_REF_LOAD: &str = "cmp_index_ref_load";
pub const INDEX_MULTIPLE_EQ: &str = "index_multiple_eq";
pub const SUB_COMPONENT_AUX: &str = "sub_component_aux";

// ─── Field-element access helpers ─────────────────────────────────────────────

pub fn lvar(at: &str) -> String {
    format!("{}[{}]", L_LVAR, at)
}

pub fn expaux(at: &str) -> String {
    format!("{}[{}]", L_EXPAUX, at)
}

pub fn signal_values(signal_start: &str, at: &str) -> String {
    format!("{}.signal_values[{} + {}]", CTX, signal_start, at)
}

pub fn circuit_constants(at: &str) -> String {
    format!("{}.circuit_constants[{}]", CTX, at)
}

pub fn subcmp_signal(cmp_idx: &str, sig_idx: &str) -> String {
    // Deferred to a two-step access: we store signal_start into _sub_start first
    format!(
        "{}.signal_values[{}.component_memory[{}[{}]].signal_start + {}]",
        CTX, CTX, MY_SUBCOMPONENTS, cmp_idx, sig_idx
    )
}

// Add an offset to an array-access expression like "lvar[5]" → "lvar[5 + _i]"
pub fn add_offset_to_expr(expr: &str, offset: &str) -> String {
    if let Some(pos) = expr.rfind('[') {
        let (prefix, rest) = expr.split_at(pos + 1);
        let inner = &rest[..rest.len() - 1]; // strip trailing ']'
        format!("{}{}+{}]", prefix, inner, offset)
    } else {
        format!("{} + {}", expr, offset)
    }
}

// ─── Code-structure helpers ───────────────────────────────────────────────────

pub fn merge_code(instructions: Vec<String>) -> String {
    format!("{}\n", instructions.join("\n"))
}

pub fn build_callable(header: &str, body: Vec<String>) -> String {
    format!("{}{{\n{}}}\n", header, merge_code(body))
}

pub fn build_call(name: &str, args: &[String]) -> String {
    format!("{}({})", name, args.join(", "))
}

pub fn build_conditional(
    cond: &str,
    if_body: Vec<String>,
    else_body: Vec<String>,
) -> String {
    let mut s = format!("if {} {{\n{}}}", cond, merge_code(if_body));
    if !else_body.is_empty() {
        s.push_str(&format!(" else {{\n{}}}", merge_code(else_body)));
    }
    s
}

// ─── Field-operation call strings ────────────────────────────────────────────

pub fn fr_op_call(op: &str, args: &[String]) -> String {
    // args are Rust lvalue expressions; we borrow them
    let borrowed: Vec<String> = args.iter().map(|a| format!("&{}", a)).collect();
    format!("fr_{}({}, &{}.prime)", op, borrowed.join(", "), CTX)
}

pub fn fr_op_call_unary(op: &str, a: &str) -> String {
    format!("fr_{}(&{}, &{}.prime)", op, a, CTX)
}

pub fn fr_is_true(expr: &str) -> String {
    format!("fr_is_true(&{})", expr)
}

pub fn fr_to_int(expr: &str) -> String {
    format!("fr_to_int(&{}, &{}.prime)", expr, CTX)
}

pub fn fr_eq_call(res: &str, a: &str, b: &str) -> String {
    format!("{} = fr_eq(&{}, &{}, &{}.prime);", res, a, b, CTX)
}

pub fn fr_zero() -> String {
    "FrElement::from(0u32)".to_string()
}

// ─── Template / function headers ─────────────────────────────────────────────

pub fn template_run_header(header: &str) -> String {
    format!("#[allow(non_snake_case)]\nfn {}_run(ctx_index: usize, ctx: &mut CircomCalcWit)", header)
}

pub fn template_create_header(header: &str) -> String {
    format!(
        "#[allow(non_snake_case)]\nfn {}_create(soffset: usize, coffset: usize, ctx: &mut CircomCalcWit, _component_name: &str, _component_father: usize)",
        header
    )
}

pub fn function_header(header: &str) -> String {
    format!(
        "#[allow(non_snake_case)]\nfn {}(ctx: &mut CircomCalcWit, lvar: &mut Vec<FrElement>, component_father: usize, destination: &mut Vec<FrElement>, destination_size: usize)",
        header
    )
}

// ─── dat-file helpers (hash map, constants, io map) ──────────────────────────

pub fn generate_hash_map(signal_name_list: &Vec<InputInfo>, size: usize) -> Vec<(u64, u64, u64)> {
    assert!(signal_name_list.len() <= size);
    let mut hash_map = vec![(0u64, 0u64, 0u64); size];
    for info in signal_name_list {
        let h = hasher(&info.name);
        let mut p = h as usize % size;
        while hash_map[p].1 != 0 {
            p = (p + 1) % size;
        }
        hash_map[p] = (h, info.start as u64, info.size as u64);
    }
    hash_map
}

// ─── Cargo.toml generation ───────────────────────────────────────────────────

pub fn generate_cargo_toml(run_name: &str) -> String {
    format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
malachite = "0.4"
rayon = "1"
"#,
        run_name
    )
}

// ─── Generated lib.rs prologue / epilogue ────────────────────────────────────

pub fn generate_lib_prologue() -> Vec<String> {
    vec![
        "// Generated by circom - DO NOT EDIT".to_string(),
        "#![allow(unused_imports, unused_mut, unused_variables, dead_code)]".to_string(),
        "mod field;".to_string(),
        "mod calcwit;".to_string(),
        "use field::*;".to_string(),
        "pub use calcwit::CircomCalcWit;".to_string(),
        "use std::collections::BTreeMap;".to_string(),
        "".to_string(),
    ]
}

pub fn generate_circuit_metadata(producer: &RustProducer) -> Vec<String> {
    let mut code = vec![
        "// ─── Circuit metadata ───────────────────────────────────────────────".to_string(),
    ];
    code.push(format!(
        "pub fn get_main_input_signal_start() -> usize {{ {} }}",
        producer.get_number_of_main_outputs()
    ));
    code.push(format!(
        "pub fn get_main_input_signal_no() -> usize {{ {} }}",
        producer.get_number_of_main_inputs()
    ));
    code.push(format!(
        "pub fn get_total_signal_no() -> usize {{ {} }}",
        producer.get_total_number_of_signals()
    ));
    code.push(format!(
        "pub fn get_number_of_components() -> usize {{ {} }}",
        producer.get_number_of_components()
    ));
    code.push(format!(
        "pub fn get_size_of_input_hashmap() -> usize {{ {} }}",
        producer.get_input_hash_map_entry_size()
    ));
    code.push(format!(
        "pub fn get_size_of_witness() -> usize {{ {} }}",
        producer.get_witness_to_signal_list().len()
    ));
    code.push(format!(
        "pub fn get_size_of_constants() -> usize {{ {} }}",
        producer.get_field_constant_list().len()
    ));
    code.push(format!(
        "pub fn get_size_of_io_map() -> usize {{ {} }}",
        producer.get_io_map().len()
    ));
    code.push(format!(
        "pub fn get_size_of_bus_field_map() -> usize {{ {} }}",
        producer.get_busid_field_info().len()
    ));
    code.push("".to_string());
    code
}

pub fn generate_constants_and_witness(producer: &RustProducer) -> Vec<String> {
    let mut code = vec![
        "// ─── Embedded constants ─────────────────────────────────────────────".to_string(),
    ];

    // Hash map
    let hash_map = generate_hash_map(
        producer.get_main_input_list(),
        producer.get_input_hash_map_entry_size(),
    );
    let mut hm_entries = vec![];
    for (h, sid, sz) in &hash_map {
        hm_entries.push(format!("({}_u64, {}_u64, {}_u64)", h, sid, sz));
    }
    code.push(format!(
        "const INPUT_HASH_MAP: &[(u64, u64, u64)] = &[{}];",
        hm_entries.join(", ")
    ));

    // Witness to signal
    let w2s: Vec<String> = producer
        .get_witness_to_signal_list()
        .iter()
        .map(|x| x.to_string())
        .collect();
    code.push(format!(
        "const WITNESS_TO_SIGNAL: &[usize] = &[{}];",
        w2s.join(", ")
    ));

    // Field constants (as decimal strings)
    let consts: Vec<String> = producer
        .get_field_constant_list()
        .iter()
        .map(|s| format!("\"{}\"", s))
        .collect();
    code.push(format!(
        "const FIELD_CONSTANTS: &[&str] = &[{}];",
        consts.join(", ")
    ));

    // Prime
    code.push(format!(
        "const PRIME: &str = \"{}\";",
        producer.get_prime()
    ));

    code.push("".to_string());
    code
}

pub fn generate_io_map_constants(producer: &RustProducer) -> Vec<String> {
    let mut code = vec![
        "// ─── IO map ─────────────────────────────────────────────────────────".to_string(),
    ];
    // Emit as a static initialiser function
    code.push("fn build_io_map() -> BTreeMap<usize, Vec<(usize, usize, Vec<usize>, Option<usize>)>> {".to_string());
    code.push("    let mut m = BTreeMap::new();".to_string());
    for (tid, io_list) in producer.get_io_map() {
        let entries: Vec<String> = io_list
            .iter()
            .map(|d| {
                let lens: Vec<String> = d.lengths.iter().map(|x| x.to_string()).collect();
                let bus_id = match d.bus_id {
                    Some(b) => format!("Some({})", b),
                    None => "None".to_string(),
                };
                format!("({}, {}, vec![{}], {})", d.offset, d.size, lens.join(", "), bus_id)
            })
            .collect();
        code.push(format!(
            "    m.insert({}, vec![{}]);",
            tid,
            entries.join(", ")
        ));
    }
    code.push("    m".to_string());
    code.push("}".to_string());

    // Bus field info
    code.push("fn build_bus_field_info() -> Vec<Vec<(usize, usize, Vec<usize>, Option<usize>)>> {".to_string());
    code.push("    let mut v = Vec::new();".to_string());
    for bus in producer.get_busid_field_info() {
        let entries: Vec<String> = bus
            .iter()
            .map(|d| {
                let dims: Vec<String> = d.dimensions.iter().map(|x| x.to_string()).collect();
                let bus_id = match d.bus_id {
                    Some(b) => format!("Some({})", b),
                    None => "None".to_string(),
                };
                format!("({}, {}, vec![{}], {})", d.offset, d.size, dims.join(", "), bus_id)
            })
            .collect();
        code.push(format!("    v.push(vec![{}]);", entries.join(", ")));
    }
    code.push("    v".to_string());
    code.push("}".to_string());

    code.push("".to_string());
    code
}

pub fn generate_function_table(producer: &RustProducer) -> Vec<String> {
    let mut code = vec![
        "// ─── Template dispatch ─────────────────────────────────────────────".to_string(),
    ];
    code.push("#[allow(non_snake_case)]".to_string());
    code.push(
        "pub fn run_template(template_id: usize, ctx_index: usize, ctx: &mut CircomCalcWit) {"
            .to_string(),
    );
    code.push("    match template_id {".to_string());
    for (i, inst) in producer.get_template_instance_list().iter().enumerate() {
        code.push(format!(
            "        {} => {}_run(ctx_index, ctx),",
            i, inst.name
        ));
    }
    code.push(
        "        _ => panic!(\"Unknown template id {}\", template_id),".to_string(),
    );
    code.push("    }".to_string());
    code.push("}".to_string());
    code.push("".to_string());
    code
}

pub fn generate_circuit_entry_and_init(producer: &RustProducer) -> Vec<String> {
    let main_header = producer.get_main_header();
    let mut code = vec![
        "// ─── Entry point ──────────────────────────────────────────────────".to_string(),
    ];
    code.push("pub fn run(ctx: &mut CircomCalcWit) {".to_string());
    if producer.main_is_parallel {
        code.push(format!(
            "    {}_create_parallel(1, 0, ctx, \"main\", 0);",
            main_header
        ));
    } else {
        code.push(format!(
            "    {}_create(1, 0, ctx, \"main\", 0);",
            main_header
        ));
    }
    // If _create set input_counter > 0, inputs are already in signal_values
    // (run() is only called from set_input_signal when remaining_input_signals hits 0).
    // Reset counter and dispatch immediately.
    code.push("    if ctx.component_memory[0].input_counter > 0 {".to_string());
    code.push("        ctx.component_memory[0].input_counter = 0;".to_string());
    code.push("        let _f = ctx.run_template;".to_string());
    code.push("        let _tid = ctx.component_memory[0].template_id;".to_string());
    code.push("        _f(_tid, 0, ctx);".to_string());
    code.push("    }".to_string());
    code.push("}".to_string());
    code.push("".to_string());

    // create_witness_calculator initialiser
    code.push(
        "/// Build and return a fresh `CircomCalcWit` ready to receive input signals.".to_string(),
    );
    code.push("pub fn create_witness_calculator() -> CircomCalcWit {".to_string());
    code.push(format!(
        "    let prime: malachite::integer::Integer = PRIME.parse().expect(\"invalid prime\");"
    ));
    code.push("    let constants: Vec<FrElement> = FIELD_CONSTANTS".to_string());
    code.push(
        "        .iter().map(|s| s.parse::<malachite::integer::Integer>().unwrap()).collect();"
            .to_string(),
    );
    code.push("    let io_map = build_io_map();".to_string());
    code.push("    let bus_field_info = build_bus_field_info();".to_string());
    code.push(
        "    let ctx = CircomCalcWit::new(".to_string(),
    );
    code.push(format!(
        "        get_total_signal_no(),\n        get_number_of_components(),"
    ));
    code.push("        constants,".to_string());
    code.push("        INPUT_HASH_MAP,".to_string());
    code.push("        WITNESS_TO_SIGNAL,".to_string());
    code.push("        get_main_input_signal_start(),".to_string());
    code.push("        get_main_input_signal_no(),".to_string());
    code.push("        prime,".to_string());
    code.push("        io_map,".to_string());
    code.push("        bus_field_info,".to_string());
    code.push("        run_template,".to_string());
    code.push("        run,".to_string());
    code.push("    );".to_string());
    code.push("    ctx".to_string());
    code.push("}".to_string());
    code
}

// ─── File-writing helpers ─────────────────────────────────────────────────────

pub fn generate_field_rs_file(rust_folder: &PathBuf) -> std::io::Result<()> {
    let contents = include_str!("common/field.rs");
    let mut path = rust_folder.clone();
    path.push("src");
    path.push("field.rs");
    let mut f = File::create(&path)?;
    f.write_all(contents.as_bytes())?;
    f.flush()
}

pub fn generate_calcwit_rs_file(rust_folder: &PathBuf) -> std::io::Result<()> {
    let contents = include_str!("common/calcwit.rs");
    let mut path = rust_folder.clone();
    path.push("src");
    path.push("calcwit.rs");
    let mut f = File::create(&path)?;
    f.write_all(contents.as_bytes())?;
    f.flush()
}

pub fn generate_batch_fn() -> Vec<String> {
    vec![
        "// ─── Batch witness computation ──────────────────────────────────────".to_string(),
        "/// Compute witnesses for a batch of inputs in parallel using rayon.".to_string(),
        "/// Each entry maps input signal name to its array of values (use a".to_string(),
        "/// single-element vec for scalar inputs).".to_string(),
        "pub fn compute_witness_batch(".to_string(),
        "    batch: Vec<std::collections::BTreeMap<String, Vec<FrElement>>>,".to_string(),
        ") -> Vec<Vec<FrElement>> {".to_string(),
        "    use rayon::prelude::*;".to_string(),
        "    batch.into_par_iter().map(|inputs| {".to_string(),
        "        let mut ctx = create_witness_calculator();".to_string(),
        "        for (name, values) in inputs {".to_string(),
        "            for (idx, val) in values.into_iter().enumerate() {".to_string(),
        "                ctx.set_input_signal(&name, idx, val);".to_string(),
        "            }".to_string(),
        "        }".to_string(),
        "        ctx.get_witness_vec()".to_string(),
        "    }).collect()".to_string(),
        "}".to_string(),
    ]
}

pub fn generate_cargo_toml_file(rust_folder: &PathBuf, run_name: &str) -> std::io::Result<()> {
    let contents = generate_cargo_toml(run_name);
    let mut path = rust_folder.clone();
    path.push("Cargo.toml");
    let mut f = File::create(&path)?;
    f.write_all(contents.as_bytes())?;
    f.flush()
}
