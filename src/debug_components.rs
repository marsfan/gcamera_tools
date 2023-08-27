#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
pub struct DebugChunk {
    pub magic: String,
    pub data: Vec<u8>,
}

impl DebugChunk {
    pub fn to_bytes(self) -> Vec<u8> {
        return [self.magic.into_bytes(), self.data].concat();
    }
}

// FIXME: Need a way to handle being out of range
fn find_magic_start(data: &[u8], magic: &[u8]) -> Result<usize, &'static str> {
    for (position, _) in data.iter().enumerate() {
        let last_byte = position + magic.len();
        let chunk = data[position..last_byte].to_vec();
        if chunk == magic {
            return Ok(position);
        }
    }
    return Err("Could not find start of magic.");
}

pub struct DebugComponents {
    pub aecdebug: DebugChunk,
    pub afdebug: DebugChunk,
    pub awbdebug: DebugChunk,
}

// TODO: Could possibly use "bytes.window" instead?
fn find_awb_end(bytes: &[u8]) -> usize {
    let magic = "\x00\x00\x00\x1cftypisom".as_bytes();
    let length = bytes.len();
    let range_end = length - magic.len();
    for (offset, _) in bytes[..range_end].iter().enumerate() {
        if &bytes[offset..offset + magic.len()] == magic {
            return offset;
        }
    }
    return bytes.len() - 1;
}

impl DebugComponents {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        // TODO: Proper Error Handling
        let aec_start = find_magic_start(bytes, b"aecDebug").unwrap();
        let af_start = find_magic_start(&bytes[aec_start..], b"afDebug").unwrap() + aec_start;
        let awb_start = find_magic_start(&bytes[af_start..], b"awbDebug").unwrap() + af_start;
        let awb_end = find_awb_end(&bytes[awb_start..]) + awb_start;

        return DebugComponents {
            aecdebug: DebugChunk {
                magic: String::from_utf8(bytes[aec_start..aec_start + 8].to_vec()).unwrap(),
                data: bytes[aec_start + 8..af_start].to_vec(),
            },
            afdebug: DebugChunk {
                magic: String::from_utf8(bytes[af_start..af_start + 7].to_vec()).unwrap(),
                data: bytes[af_start + 7..awb_start].to_vec(),
            },
            awbdebug: DebugChunk {
                magic: String::from_utf8(bytes[awb_start..awb_start + 8].to_vec()).unwrap(),
                data: bytes[awb_start + 8..awb_end].to_vec(),
            },
        };
    }

    pub fn to_bytes(self) -> Vec<u8> {
        return [
            self.aecdebug.to_bytes(),
            self.afdebug.to_bytes(),
            self.awbdebug.to_bytes(),
        ]
        .concat();
    }
}
