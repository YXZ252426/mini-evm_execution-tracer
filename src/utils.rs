use eyre::{eyre, Result};
use std::{fs, path::Path};

pub fn read_hex_file(path: &Path) -> Result<Vec<u8>> {
    let raw = fs::read_to_string(path)?;
    parse_hex(&raw)
}

pub fn parse_hex(input: &str) -> Result<Vec<u8>> {
    let s = input.trim().strip_prefix("0x").unwrap_or(input.trim());

    if s.is_empty() {
        return Ok(vec![]);
    }

    if s.len() % 2 != 0 {
        return Err(eyre!("hex string length must be even"));
    }

    Ok(hex::decode(s)?)
}

pub fn opcode_name(opcode: u8) -> String {
    match opcode {
        0x00 => "STOP",
        0x01 => "ADD",
        0x02 => "MUL",
        0x03 => "SUB",
        0x04 => "DIV",
        0x05 => "LT",
        0x06 => "GT",
        0x10 => "EQ",
        0x11 => "ISZERO",
        0x14 => "EQ",
        0x15 => "ISZERO",
        0x20 => "SHA3",
        0x30 => "ADDRESS",
        0x31 => "BALANCE",
        0x33 => "CALLER",
        0x34 => "CALLVALUE",
        0x35 => "CALLDATALOAD",
        0x36 => "CALLDATASIZE",
        0x37 => "CALLDATACOPY",
        0x39 => "CODECOPY",
        0x51 => "MLOAD",
        0x52 => "MSTORE",
        0x54 => "SLOAD",
        0x55 => "SSTORE",
        0x56 => "JUMP",
        0x57 => "JUMPI",
        0x5b => "JUMPDEST",
        0xf1 => "CALL",
        0xf3 => "RETURN",
        0xfd => "REVERT",
        0xff => "SELFDESTRUCT",
        0x60..=0x7f => return format!("PUSH{}", opcode - 0x5f),
        0x80..=0x8f => return format!("DUP{}", opcode - 0x7f),
        0x90..=0x9f => return format!("SWAP{}", opcode - 0x8f),
        0xa0..=0xa4 => return format!("LOG{}", opcode - 0xa0),
        _ => "UNKNOWN",
    }
    .to_string()
}
