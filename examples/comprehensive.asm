// Comprehensive tracer demo bytecode.
//
// Covers:
// - memory write and memory preview: MSTORE stores 0x2a at memory[0..32]
// - storage writes: slot 0 = 1, slot 2 = 0xbeef
// - storage read: SLOAD slot 0
// - event log: LOG1 emits memory[0..32] with topic 0x1111...
// - nested call trace: CALL precompile address 0x01
//
// Hex:
// 602a600052
// 6001600055
// 61beef600255
// 60005450
// 7f1111111111111111111111111111111111111111111111111111111111111111
// 60206000a1
// 6000600060006000600060006001612710f1
// 00

PUSH1 0x2a
PUSH1 0x00
MSTORE

PUSH1 0x01
PUSH1 0x00
SSTORE

PUSH2 0xbeef
PUSH1 0x02
SSTORE

PUSH1 0x00
SLOAD
POP

PUSH32 0x1111111111111111111111111111111111111111111111111111111111111111
PUSH1 0x20
PUSH1 0x00
LOG1

PUSH1 0x00 // out_size
PUSH1 0x00 // out_offset
PUSH1 0x00 // in_size
PUSH1 0x00 // in_offset
PUSH1 0x00 // value
PUSH1 0x01 // to: ecrecover precompile
PUSH2 0x2710 // gas
CALL

STOP
