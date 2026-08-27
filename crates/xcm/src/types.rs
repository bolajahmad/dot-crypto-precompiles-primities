use std::collections::BTreeMap;

/// Version-agnostic view of a single XCM instruction.
#[derive(Debug, Default, Clone)]
pub struct XcmInstruction {
    pub name: String,
    pub params: BTreeMap<String, Vec<String>>,
}

impl XcmInstruction {
    pub fn new(name: impl Into<String>, params: BTreeMap<String, Vec<String>>) -> Self {
        Self {
            name: name.into(),
            params,
        }
    }

    pub fn note(name: impl Into<String>, note: impl Into<String>) -> Self {
        let mut params = BTreeMap::new();
        params.insert("Note".to_string(), vec![note.into()]);
        Self::new(name, params)
    }
}

/// Decoded XCM message summary shared across all versions.
#[derive(Debug, Default, Clone)]
pub struct XcmMessage {
    /// XCM version (3 | 4 | 5).
    version: u32,
    /// Number of top-level instructions.
    size: usize,
    /// Encoded byte length of the versioned message.
    bytes: usize,
    instructions: Vec<XcmInstruction>,
}

impl XcmMessage {
    pub fn new(version: u32, size: usize, bytes: usize, instructions: Vec<XcmInstruction>) -> Self {
        Self {
            version,
            size,
            bytes,
            instructions,
        }
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn instructions(&self) -> Vec<XcmInstruction> {
        self.instructions.clone()
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn bytes(&self) -> usize {
        self.bytes
    }
}

#[derive(Default, Debug)]
pub struct CError {
    pub error: String,
    pub message: String,
}

impl CError {
    pub fn new(err: String, message: &str) -> Self {
        Self {
            error: err,
            message: message.to_string(),
        }
    }
}

/// Parse a versioned XCM program into shared instruction views.
pub trait XcmParser {
    fn parse_message(&self) -> Vec<XcmInstruction>;
}
