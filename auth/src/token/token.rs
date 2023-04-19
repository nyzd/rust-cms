use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct TokenGenerator<'a> {
    /// Target data
    source: &'a Vec<u8>,

    /// Final Generated Token
    result: Option<String>,
}

impl<'a> TokenGenerator<'a> {
    /// Creates a new Token object
    pub fn new(source: &'a Vec<u8>) -> Self {
        Self {
            source,
            result: None,
        }
    }

    /// Change the source
    pub fn set_source(&mut self, new_source: &'a Vec<u8>) {
        self.source = new_source;
    }

    /// Generates the final hash and set to result
    pub fn generate(&mut self) {
        let mut hasher = Sha256::new();

        hasher.update(self.source);

        self.result = Some(format!("{:x}", hasher.finalize()));
    }

    /// Returns the copy of result
    pub fn get_result(&self) -> Option<String> {
        self.result.clone()
    }
}
