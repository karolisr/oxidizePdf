//! RC4 encryption algorithm implementation

/// RC4 key for encryption/decryption
#[derive(Debug, Clone)]
pub struct Rc4Key {
    /// Key bytes
    pub key: Vec<u8>,
}

impl Rc4Key {
    /// Create a new RC4 key
    pub fn new(key: Vec<u8>) -> Self {
        Self { key }
    }

    /// Create from slice
    pub fn from_slice(key: &[u8]) -> Self {
        Self { key: key.to_vec() }
    }
}

/// RC4 cipher state
pub struct Rc4 {
    /// State array
    s: [u8; 256],
    /// Index i
    i: usize,
    /// Index j
    j: usize,
}

impl Rc4 {
    /// Create a new RC4 cipher with the given key
    pub fn new(key: &Rc4Key) -> Self {
        let mut s = [0u8; 256];

        // Initialize state array
        for (i, byte) in s.iter_mut().enumerate() {
            *byte = i as u8;
        }

        // Key scheduling algorithm (KSA)
        let mut j = 0usize;
        for i in 0..256 {
            j = (j + s[i] as usize + key.key[i % key.key.len()] as usize) % 256;
            s.swap(i, j);
        }

        Self { s, i: 0, j: 0 }
    }

    /// Process data (encrypt or decrypt - RC4 is symmetric)
    pub fn process(&mut self, data: &[u8]) -> Vec<u8> {
        let mut output = Vec::with_capacity(data.len());

        for &byte in data {
            // Pseudo-random generation algorithm (PRGA)
            self.i = (self.i + 1) % 256;
            self.j = (self.j + self.s[self.i] as usize) % 256;
            self.s.swap(self.i, self.j);

            let k = self.s[(self.s[self.i] as usize + self.s[self.j] as usize) % 256];
            output.push(byte ^ k);
        }

        output
    }

    /// Process data in place
    pub fn process_in_place(&mut self, data: &mut [u8]) {
        for byte in data.iter_mut() {
            // PRGA
            self.i = (self.i + 1) % 256;
            self.j = (self.j + self.s[self.i] as usize) % 256;
            self.s.swap(self.i, self.j);

            let k = self.s[(self.s[self.i] as usize + self.s[self.j] as usize) % 256];
            *byte ^= k;
        }
    }
}

/// Encrypt data using RC4
#[allow(dead_code)]
pub fn rc4_encrypt(key: &Rc4Key, data: &[u8]) -> Vec<u8> {
    let mut cipher = Rc4::new(key);
    cipher.process(data)
}

/// Decrypt data using RC4 (same as encrypt for RC4)
#[allow(dead_code)]
pub fn rc4_decrypt(key: &Rc4Key, data: &[u8]) -> Vec<u8> {
    rc4_encrypt(key, data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rc4_key_creation() {
        let key = Rc4Key::new(vec![1, 2, 3, 4, 5]);
        assert_eq!(key.key, vec![1, 2, 3, 4, 5]);

        let key2 = Rc4Key::from_slice(&[6, 7, 8]);
        assert_eq!(key2.key, vec![6, 7, 8]);
    }

    #[test]
    fn test_rc4_encryption_decryption() {
        let key = Rc4Key::new(vec![0x01, 0x02, 0x03, 0x04, 0x05]);
        let plaintext = b"Hello, World!";

        // Encrypt
        let ciphertext = rc4_encrypt(&key, plaintext);
        assert_ne!(ciphertext, plaintext);

        // Decrypt
        let decrypted = rc4_decrypt(&key, &ciphertext);
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_rc4_process_in_place() {
        let key = Rc4Key::new(vec![0x01, 0x02, 0x03, 0x04, 0x05]);
        let mut data = b"Test data".to_vec();
        let original = data.clone();

        // Encrypt in place
        let mut cipher = Rc4::new(&key);
        cipher.process_in_place(&mut data);
        assert_ne!(data, original);

        // Decrypt in place
        let mut cipher = Rc4::new(&key);
        cipher.process_in_place(&mut data);
        assert_eq!(data, original);
    }

    #[test]
    fn test_rc4_known_vectors() {
        // Test vector from RFC 6229
        let key = Rc4Key::from_slice(&[0x01, 0x02, 0x03, 0x04, 0x05]);
        let mut cipher = Rc4::new(&key);

        // Generate keystream
        let zeros = vec![0u8; 16];
        let keystream = cipher.process(&zeros);

        // First 16 bytes of keystream for key [01 02 03 04 05]
        let expected = [0xb2, 0x39, 0x63, 0x05, 0xf0, 0x3d, 0xc0, 0x27, 0xcc, 0xc3, 0x52, 0x4a, 0x0a, 0x11,
            0x18, 0xa8];

        assert_eq!(&keystream[..16], &expected[..]);
    }
}
