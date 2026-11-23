#![no_std]

pub mod bus;
pub mod can;
pub mod ecu;
pub mod error;
pub mod quad;
pub mod signal;

pub use bus::{CanBus, CanFilter, EcuRegistry, QuadSequenceTracker};
pub use can::{CanFrame, CanId, FrameFlags};
pub use ecu::{EcuCommand, EcuNode, EcuState};
pub use error::AutomotiveError;
pub use quad::{
    get_quad, max_quad, read_quads, set_quad, write_quads, Quad, QuadArray,
};
pub use signal::{dequantize_quad_to_u8, quantize_u8_to_quad, QuadSignal};

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::format;
    use std::println;
    use std::time::Instant;
    use std::vec;
    use std::vec::Vec;

    struct BenchmarkReport {
        name: &'static str,
        iterations: u128,
        elapsed_ns: u128,
        avg_ns_per_iter: f64,
    }

    impl BenchmarkReport {
        fn print(&self) {
            println!(
                "[BENCH] {:<42} | iters: {:>8} | total: {:>10} ns | avg/iter: {:>10.2} ns",
                self.name, self.iterations, self.elapsed_ns, self.avg_ns_per_iter
            );
        }
    }

    fn run_bench<F>(name: &'static str, iterations: u128, mut work: F)
    where
        F: FnMut(),
    {
        let start = Instant::now();
        for _ in 0..iterations {
            work();
        }
        let elapsed_ns = start.elapsed().as_nanos();
        let avg_ns_per_iter = if iterations > 0 {
            (elapsed_ns as f64) / (iterations as f64)
        } else {
            0.0
        };
        BenchmarkReport {
            name,
            iterations,
            elapsed_ns,
            avg_ns_per_iter,
        }
        .print();
    }

    struct MockCanBus {
        tx_queue: Vec<CanFrame>,
        rx_queue: Vec<CanFrame>,
        ready: bool,
    }

    impl MockCanBus {
        fn new(ready: bool) -> Self {
            Self {
                tx_queue: Vec::new(),
                rx_queue: Vec::new(),
                ready,
            }
        }
    }

    impl CanBus for MockCanBus {
        fn transmit(&mut self, frame: &CanFrame) -> Result<(), AutomotiveError> {
            if !self.ready {
                return Err(AutomotiveError::BusNotReady);
            }
            self.tx_queue.push(*frame);
            Ok(())
        }

        fn receive(&mut self) -> Result<CanFrame, AutomotiveError> {
            if !self.ready {
                return Err(AutomotiveError::BusNotReady);
            }
            self.rx_queue.pop().ok_or(AutomotiveError::BusReceiveEmpty)
        }

        fn is_ready(&self) -> bool {
            self.ready
        }
    }

    // 1. Easiest: Basic Quad bit conversion and bool-pair roundtrip
    #[test]
    fn test_01_quad_basic_conversion() {
        let iters = 200_000;
        let mut sink = 0u8;

        run_bench("01_quad_basic_conversion", iters, || {
            for b in 0u8..4 {
                let q = Quad::from_bits(b);
                let (hi, lo) = q.to_bool_pair();
                let rebuilt = Quad::from_bool_pair(hi, lo);
                assert_eq!(q, rebuilt);
                sink ^= rebuilt.as_bits();
            }
        });

        assert!(sink < 4);
    }

    // 2. Easy: Single get_quad and set_quad packing in a byte buffer
    #[test]
    fn test_02_single_get_set_quad() {
        let iters = 100_000;
        let mut buf = [0u8; 4];

        run_bench("02_single_get_set_quad", iters, || {
            set_quad(&mut buf, 0, Quad::Q00).unwrap();
            set_quad(&mut buf, 1, Quad::Q01).unwrap();
            set_quad(&mut buf, 2, Quad::Q10).unwrap();
            set_quad(&mut buf, 3, Quad::Q11).unwrap();

            assert_eq!(get_quad(&buf, 0).unwrap(), Quad::Q00);
            assert_eq!(get_quad(&buf, 1).unwrap(), Quad::Q01);
            assert_eq!(get_quad(&buf, 2).unwrap(), Quad::Q10);
            assert_eq!(get_quad(&buf, 3).unwrap(), Quad::Q11);
        });
    }

    // 3. Easy-Medium: Standard and Extended CanId validation and priority
    #[test]
    fn test_03_can_id_validation_and_priority() {
        let iters = 100_000;

        run_bench("03_can_id_validation_and_priority", iters, || {
            let std_id = CanId::standard(0x123).unwrap();
            let ext_id = CanId::extended(0x18FF_A100).unwrap();

            assert!(!std_id.is_extended());
            assert!(ext_id.is_extended());
            assert_eq!(std_id.raw(), 0x123);
            assert_eq!(ext_id.raw(), 0x18FF_A100);
            assert!(CanId::standard(0x800).is_err());
            assert!(CanId::extended(0x2000_0000).is_err());
            let _ = std_id.priority_level();
            let _ = ext_id.priority_level();
        });
    }

    // 4. Easy-Medium: CanFrame payload creation and slicing
    #[test]
    fn test_04_can_frame_payload_handling() {
        let iters = 100_000;
        let id = CanId::standard(0x321).unwrap();
        let data = [10u8, 20, 30, 40, 50, 60];

        run_bench("04_can_frame_payload_handling", iters, || {
            let frame = CanFrame::new(id, &data).unwrap();
            assert_eq!(frame.dlc, 6);
            assert_eq!(frame.payload(), &data);
            assert!(CanFrame::new(id, &[0u8; 9]).is_err());
        });
    }

    // 5. Medium: Batch read_quads, write_quads, and max_quad
    #[test]
    fn test_05_batch_quad_operations() {
        let iters = 50_000;
        let mut buf = [0u8; 8];
        let input = [
            Quad::Q00,
            Quad::Q01,
            Quad::Q10,
            Quad::Q11,
            Quad::Q01,
            Quad::Q10,
            Quad::Q00,
            Quad::Q11,
        ];
        let mut output = [Quad::Q00; 8];

        run_bench("05_batch_quad_operations", iters, || {
            write_quads(&mut buf, 0, &input).unwrap();
            read_quads(&buf, 0, &mut output).unwrap();
            assert_eq!(output, input);
            assert_eq!(max_quad(&buf, 0, 8).unwrap(), Quad::Q11);
        });
    }

    // 6. Medium: QuadArray fixed-byte packing and byte roundtrip
    #[test]
    fn test_06_quad_array_packing() {
        let iters = 50_000;

        run_bench("06_quad_array_packing", iters, || {
            let mut arr = QuadArray::<4>::new(Quad::Q01);
            arr.set(0, Quad::Q11).unwrap();
            arr.set(7, Quad::Q10).unwrap();
            arr.set(15, Quad::Q00).unwrap();

            assert_eq!(arr.get(0).unwrap(), Quad::Q11);
            assert_eq!(arr.get(7).unwrap(), Quad::Q10);
            assert_eq!(arr.get(15).unwrap(), Quad::Q00);

            let bytes = *arr.as_bytes();
            let restored = QuadArray::<4>::from_bytes(&bytes).unwrap();
            assert_eq!(arr, restored);
        });
    }

    // 7. Medium-Hard: QuadSignal encoding, decoding, and quantization
    #[test]
    fn test_07_quad_signal_encode_decode_quantize() {
        let iters = 50_000;
        let id = CanId::standard(0x456).unwrap();
        let sig = QuadSignal::new(0, 8).unwrap();
        let values = [
            quantize_u8_to_quad(0),
            quantize_u8_to_quad(90),
            quantize_u8_to_quad(180),
            quantize_u8_to_quad(250),
            Quad::Q01,
            Quad::Q10,
            Quad::Q11,
            Quad::Q00,
        ];

        run_bench("07_quad_signal_encode_decode_quantize", iters, || {
            let mut frame = CanFrame::empty(id);
            sig.encode_from_slice(&mut frame, &values).unwrap();

            let mut decoded = [Quad::Q00; 8];
            sig.decode_into_slice(&frame, &mut decoded).unwrap();
            assert_eq!(decoded, values);

            let arr: QuadArray<2> = sig.decode_array(&frame).unwrap();
            assert_eq!(arr.get(2).unwrap(), Quad::Q10);
            let _ = dequantize_quad_to_u8(arr.get(2).unwrap());
        });
    }

    // 8. Medium-Hard: EcuNode states, faults, commands, and status frame roundtrip
    #[test]
    fn test_08_ecu_node_lifecycle_and_frames() {
        let iters = 50_000;
        let id = CanId::standard(0x500).unwrap();

        run_bench("08_ecu_node_lifecycle_and_frames", iters, || {
            let mut ecu = EcuNode::new(id);
            ecu.apply_command(0, EcuCommand::Wake).unwrap();
            ecu.apply_command(0, EcuCommand::Run).unwrap();
            assert_eq!(ecu.get_slot(0).unwrap(), EcuState::Active);

            ecu.set_fault_level(1, Quad::Q11).unwrap();
            assert_eq!(ecu.get_slot(1).unwrap(), EcuState::Fault);
            assert!(ecu.has_any_fault());

            let frame = ecu.to_status_frame().unwrap();
            assert_eq!(frame.dlc, 2);
            assert_eq!(frame.payload()[0], ecu.states.as_bytes()[0]);
            assert_eq!(frame.payload()[1], ecu.faults.as_bytes()[0]);

            ecu.apply_command(1, EcuCommand::ResetFaults).unwrap();
            assert_eq!(ecu.get_slot(1).unwrap(), EcuState::Standby);
        });
    }

    // 9. Hard: CanFilter mask matching and QuadSequenceTracker wrap-around
    #[test]
    fn test_09_filter_and_sequence_tracker() {
        let iters = 50_000;
        let filter = CanFilter {
            id_filter: 0x700,
            id_mask: 0x7F0,
        };
        let matching_frame = CanFrame::empty(CanId::standard(0x705).unwrap());
        let non_matching_frame = CanFrame::empty(CanId::standard(0x720).unwrap());

        run_bench("09_filter_and_sequence_tracker", iters, || {
            assert!(filter.matches(&matching_frame));
            assert!(!filter.matches(&non_matching_frame));

            let mut seq = QuadSequenceTracker::new();
            assert_eq!(seq.advance(Quad::Q00).unwrap(), Quad::Q00);
            assert_eq!(seq.advance(Quad::Q01).unwrap(), Quad::Q01);
            assert_eq!(seq.advance(Quad::Q10).unwrap(), Quad::Q10);
            assert_eq!(seq.advance(Quad::Q11).unwrap(), Quad::Q11);
            assert_eq!(seq.advance(Quad::Q00).unwrap(), Quad::Q00);
            assert_eq!(seq.advance(Quad::Q11), Err(AutomotiveError::SequenceMismatch));
            seq.reset();
            assert_eq!(seq.expected(), Quad::Q00);
        });
    }

    // 10. Hard: EcuRegistry frame ingestion and bus broadcast
    #[test]
    fn test_10_ecu_registry_ingest_and_broadcast() {
        let iters = 25_000;

        run_bench("10_ecu_registry_ingest_and_broadcast", iters, || {
            let mut nodes = [
                EcuNode::new(CanId::standard(0x100).unwrap()),
                EcuNode::new(CanId::standard(0x101).unwrap()),
                EcuNode::new(CanId::standard(0x102).unwrap()),
                EcuNode::new(CanId::standard(0x103).unwrap()),
            ];
            let mut reg = EcuRegistry::new(&mut nodes);

            let mut incoming = EcuNode::new(CanId::standard(0x102).unwrap());
            incoming.set_slot(2, EcuState::Active).unwrap();
            let rx_frame = incoming.to_status_frame().unwrap();

            let target = reg.find_mut(0x102).unwrap();
            target.states = QuadArray::<4>::from_bytes(&[
                rx_frame.payload()[0],
                target.states.as_bytes()[1],
                target.states.as_bytes()[2],
                target.states.as_bytes()[3],
            ])
            .unwrap();
            assert_eq!(
                reg.find_mut(0x102).unwrap().get_slot(2).unwrap(),
                EcuState::Active
            );

            let mut bus = MockCanBus::new(true);
            let sent = reg.broadcast_statuses(&mut bus).unwrap();
            assert_eq!(sent, 4);
            assert_eq!(bus.tx_queue.len(), 4);
        });
    }

    // 11. Stress: High-volume multi-ECU fleet simulation with interleaved signals and faults
    #[test]
    fn test_11_stress_multi_ecu_fleet_simulation() {
        let iters = 5_000;
        let sig = QuadSignal::new(0, 16).unwrap();

        run_bench("11_stress_multi_ecu_fleet_simulation", iters, || {
            let mut fleet = [EcuNode::new(CanId::standard(0x200).unwrap()); 16];
            for (idx, node) in fleet.iter_mut().enumerate() {
                node.id = CanId::standard(0x200 + idx as u16).unwrap();
            }

            let mut bus = MockCanBus::new(true);
            for step in 0..16 {
                let mut quad_stream = [Quad::Q00; 16];
                for k in 0..16 {
                    quad_stream[k] = Quad::from_bits(((step + k) & 0b11) as u8);
                }

                let mut sig_frame = CanFrame::empty(CanId::standard(0x600).unwrap());
                sig.encode_from_slice(&mut sig_frame, &quad_stream).unwrap();
                bus.transmit(&sig_frame).unwrap();

                let mut decoded_stream = [Quad::Q00; 16];
                sig.decode_into_slice(&sig_frame, &mut decoded_stream)
                    .unwrap();
                assert_eq!(decoded_stream, quad_stream);

                let node = &mut fleet[step];
                for slot in 0..4 {
                    let state = EcuState::from(decoded_stream[slot]);
                    node.set_slot(slot, state).unwrap();
                    if state == EcuState::Fault {
                        node.set_fault_level(slot, Quad::Q11).unwrap();
                    }
                }
                let status_frame = node.to_status_frame().unwrap();
                bus.transmit(&status_frame).unwrap();
            }

            assert_eq!(bus.tx_queue.len(), 32);
        });
    }

    // 12. Maximum Stress: Full error-path, boundary, and heavy buffer torture test
    #[test]
    fn test_12_stress_boundaries_errors_and_heavy_buffer() {
        let iters = 2_000;

        run_bench("12_stress_boundaries_errors_and_heavy_buffer", iters, || {
            let mut large_buf = vec![0u8; 256];
            let total_quads = large_buf.len() * 4;

            for i in 0..total_quads {
                let q = Quad::from_bits((i & 0b11) as u8);
                set_quad(&mut large_buf, i, q).unwrap();
            }
            for i in 0..total_quads {
                let expected = Quad::from_bits((i & 0b11) as u8);
                assert_eq!(get_quad(&large_buf, i).unwrap(), expected);
            }

            // Boundary and error checks across modules
            assert_eq!(
                get_quad(&large_buf, total_quads),
                Err(AutomotiveError::IndexOutOfBounds)
            );
            assert_eq!(
                set_quad(&mut large_buf, total_quads, Quad::Q01),
                Err(AutomotiveError::IndexOutOfBounds)
            );
            assert_eq!(
                QuadSignal::new(63, 2),
                Err(AutomotiveError::BitRangeOutOfBounds)
            );
            assert_eq!(
                QuadSignal::new(0, 0),
                Err(AutomotiveError::InvalidBitWidth)
            );

            let mut ecu = EcuNode::new(CanId::extended(0x1FFF_FFFF).unwrap());
            assert_eq!(ecu.get_slot(4), Err(AutomotiveError::SlotOutOfBounds));
            assert_eq!(
                ecu.set_slot(4, EcuState::Active),
                Err(AutomotiveError::SlotOutOfBounds)
            );

            let mut offline_bus = MockCanBus::new(false);
            let dummy = CanFrame::empty(CanId::standard(0x001).unwrap());
            assert_eq!(
                offline_bus.transmit(&dummy),
                Err(AutomotiveError::BusNotReady)
            );
            assert_eq!(
                offline_bus.receive(),
                Err(AutomotiveError::BusNotReady)
            );

            let mut online_bus = MockCanBus::new(true);
            assert_eq!(
                online_bus.receive(),
                Err(AutomotiveError::BusReceiveEmpty)
            );

            let label = format!("done-{}", total_quads);
            assert!(label.starts_with("done-"));
        });
    }
}