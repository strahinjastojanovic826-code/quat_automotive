# quat_automotive

A `no_std` Rust library for automotive ECU and CAN bus communication built around quaternary logic (`00`, `01`, `10`, `11` — 2 bits per symbol) for compact state, fault, signal, and sequence representation without heap allocations[cite: 1, 2].

## Features

- **Quaternary primitives (`quad`)**: 4-state `Quad` symbols (`Q00 = 0b00`, `Q01 = 0b01`, `Q10 = 0b10`, `Q11 = 0b11`), bit-level packing/unpacking (`get_quad`, `set_quad`, `read_quads`, `write_quads`, `max_quad`), and fixed-byte `QuadArray<const PACKED_BYTES: usize>` containers.
- **CAN framing & IDs (`can`)**: Standard (11-bit) and extended (29-bit) `CanId` validation, arbitration priority helpers, mask matching, `FrameFlags`, and `CanFrame` payloads up to 8 bytes.
- **Signal encoding & quantization (`signal`)**: `QuadSignal` for packing and decoding quaternary arrays inside `CanFrame` payloads, plus `quantize_u8_to_quad` and `dequantize_quad_to_u8`.
- **ECU modeling (`ecu`)**: `EcuState`, `EcuCommand`, and `EcuNode` tracking slot states and fault levels with status frame serialization.
- **Bus & registry (`bus`)**: `CanBus` trait abstraction, `CanFilter`, `EcuRegistry` for lookup and status broadcast, and `QuadSequenceTracker` for 4-step (`00` -> `01` -> `10` -> `11`) sequence tracking.
- **Error handling (`error`)**: Explicit `AutomotiveError` variants for `no_std` operation.

## Licensing

This project is dual-licensed under:

1. **GNU Affero General Public License v3.0 (AGPL-3.0)** for open-source use.
2. **Commercial License** for proprietary/commercial applications.

### Commercial License
If you wish to use `quat_automotive` in closed-source software, proprietary products, or environments where AGPL-3.0 compliance is not feasible (e.g., embedding DLLs/shared libraries without disclosing host source code), a commercial license is required.

For commercial licensing inquiries, please contact: `strahinjastojanovic826@gmail.com`

## 📊 Benchmark

| Benchmark Name | Tier | Iterations | Total Time (ns) | Avg / Iter (ns) | Status |
| :--- | :--- | ---: | ---: | ---: | :---: |
| `04_can_frame_payload_handling` | Easy-Medium | 100,000 | 8,062,000 | 80.62 | ok |
| `03_can_id_validation_and_priority` | Easy-Medium | 100,000 | 10,410,400 | 104.10 | ok |
| `01_quad_basic_conversion` | Easiest | 200,000 | 31,886,100 | 159.43 | ok |
| `09_filter_and_sequence_tracker` | Hard | 50,000 | 8,490,400 | 169.81 | ok |
| `02_single_get_set_quad` | Easy | 100,000 | 22,721,200 | 227.21 | ok |
| `06_quad_array_packing` | Medium | 50,000 | 13,951,600 | 279.03 | ok |
| `08_ecu_node_lifecycle_and_frames` | Medium-Hard | 50,000 | 26,399,700 | 527.99 | ok |
| `05_batch_quad_operations` | Medium | 50,000 | 51,993,300 | 1,039.87 | ok |
| `10_ecu_registry_ingest_and_broadcast` | Hard | 25,000 | 30,490,500 | 1,219.62 | ok |
| `07_quad_signal_encode_decode_quantize` | Medium-Hard | 50,000 | 70,520,800 | 1,410.42 | ok |
| `12_stress_boundaries_errors_and_heavy_buffer` | Maximum Stress | 2,000 | 140,947,000 | 70,473.50 | ok |
| `11_stress_multi_ecu_fleet_simulation` | Stress | 5,000 | 159,450,500 | 31,890.10 | ok |

## Project Structure

```text
quat_automotive/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── bus.rs
    ├── can.rs
    ├── ecu.rs
    ├── error.rs
    ├── quad.rs
    └── signal.rs
```

## Usage Example

```rust
use quat_automotive::{
    CanFrame, CanId, EcuCommand, EcuNode, EcuState, Quad, QuadArray, QuadSignal,
};

let id = CanId::standard(0x500).unwrap();
let mut ecu = EcuNode::new(id);

ecu.apply_command(0, EcuCommand::Wake).unwrap();
ecu.apply_command(0, EcuCommand::Run).unwrap();
assert_eq!(ecu.get_slot(0).unwrap(), EcuState::Active);

let sig = QuadSignal::new(0, 4).unwrap();
let values = [Quad::Q00, Quad::Q01, Quad::Q10, Quad::Q11];
let mut frame = CanFrame::empty(id);
sig.encode_from_slice(&mut frame, &values).unwrap();

let packed: QuadArray<1> = sig.decode_array(&frame).unwrap();
assert_eq!(packed.get(3).unwrap(), Quad::Q11);
```

## Building and Running Tests

Check `no_std` compilation:

```bash
cargo check
```

Run the test suite with benchmark output:

```bash
cargo test -- --nocapture
```

```bash
cargo test 
```