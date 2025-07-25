//! PDF encryption dictionary structures

use crate::encryption::Permissions;
use crate::objects::{Dictionary, Object};

/// Encryption algorithm
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncryptionAlgorithm {
    /// RC4 encryption
    RC4,
    /// AES encryption (128-bit)
    AES128,
    /// AES encryption (256-bit)
    AES256,
}

/// Crypt filter method
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CryptFilterMethod {
    /// No encryption
    None,
    /// RC4 encryption
    V2,
    /// AES encryption
    AESV2,
    /// AES-256 encryption
    AESV3,
}

impl CryptFilterMethod {
    /// Get PDF name
    pub fn pdf_name(&self) -> &'static str {
        match self {
            CryptFilterMethod::None => "None",
            CryptFilterMethod::V2 => "V2",
            CryptFilterMethod::AESV2 => "AESV2",
            CryptFilterMethod::AESV3 => "AESV3",
        }
    }
}

/// Stream filter name
#[derive(Debug, Clone)]
pub enum StreamFilter {
    /// Identity (no encryption)
    Identity,
    /// Standard encryption
    StdCF,
    /// Custom filter
    Custom(String),
}

/// String filter name
#[derive(Debug, Clone)]
pub enum StringFilter {
    /// Identity (no encryption)
    Identity,
    /// Standard encryption
    StdCF,
    /// Custom filter
    Custom(String),
}

/// Crypt filter definition
#[derive(Debug, Clone)]
pub struct CryptFilter {
    /// Filter name
    pub name: String,
    /// Encryption method
    pub method: CryptFilterMethod,
    /// Length in bytes (for RC4)
    pub length: Option<u32>,
}

impl CryptFilter {
    /// Create standard crypt filter
    pub fn standard(method: CryptFilterMethod) -> Self {
        Self {
            name: "StdCF".to_string(),
            method,
            length: match method {
                CryptFilterMethod::V2 => Some(16), // 128-bit
                _ => None,
            },
        }
    }

    /// Convert to dictionary
    pub fn to_dict(&self) -> Dictionary {
        let mut dict = Dictionary::new();

        dict.set("CFM", Object::Name(self.method.pdf_name().to_string()));

        if let Some(length) = self.length {
            dict.set("Length", Object::Integer(length as i64));
        }

        dict
    }
}

/// PDF encryption dictionary
#[derive(Debug, Clone)]
pub struct EncryptionDictionary {
    /// Filter (always "Standard" for standard security handler)
    pub filter: String,
    /// Sub-filter (for public-key security handlers)
    pub sub_filter: Option<String>,
    /// Algorithm version (1-5)
    pub v: u32,
    /// Key length in bytes
    pub length: Option<u32>,
    /// Crypt filters
    pub cf: Option<Vec<CryptFilter>>,
    /// Stream filter
    pub stm_f: Option<StreamFilter>,
    /// String filter
    pub str_f: Option<StringFilter>,
    /// Identity filter
    pub ef: Option<String>,
    /// Revision number
    pub r: u32,
    /// Owner password hash (32 bytes)
    pub o: Vec<u8>,
    /// User password hash (32 bytes)
    pub u: Vec<u8>,
    /// Permissions
    pub p: Permissions,
    /// Whether to encrypt metadata
    pub encrypt_metadata: bool,
    /// Document ID (first element)
    pub id: Option<Vec<u8>>,
}

impl EncryptionDictionary {
    /// Create RC4 40-bit encryption dictionary
    pub fn rc4_40bit(
        owner_hash: Vec<u8>,
        user_hash: Vec<u8>,
        permissions: Permissions,
        id: Option<Vec<u8>>,
    ) -> Self {
        Self {
            filter: "Standard".to_string(),
            sub_filter: None,
            v: 1,
            length: Some(5), // 40 bits = 5 bytes
            cf: None,
            stm_f: None,
            str_f: None,
            ef: None,
            r: 2,
            o: owner_hash,
            u: user_hash,
            p: permissions,
            encrypt_metadata: true,
            id,
        }
    }

    /// Create RC4 128-bit encryption dictionary
    pub fn rc4_128bit(
        owner_hash: Vec<u8>,
        user_hash: Vec<u8>,
        permissions: Permissions,
        id: Option<Vec<u8>>,
    ) -> Self {
        Self {
            filter: "Standard".to_string(),
            sub_filter: None,
            v: 2,
            length: Some(16), // 128 bits = 16 bytes
            cf: None,
            stm_f: None,
            str_f: None,
            ef: None,
            r: 3,
            o: owner_hash,
            u: user_hash,
            p: permissions,
            encrypt_metadata: true,
            id,
        }
    }

    /// Convert to PDF dictionary
    pub fn to_dict(&self) -> Dictionary {
        let mut dict = Dictionary::new();

        dict.set("Filter", Object::Name(self.filter.clone()));

        if let Some(ref sub_filter) = self.sub_filter {
            dict.set("SubFilter", Object::Name(sub_filter.clone()));
        }

        dict.set("V", Object::Integer(self.v as i64));

        if let Some(length) = self.length {
            dict.set("Length", Object::Integer((length * 8) as i64)); // Convert bytes to bits
        }

        dict.set("R", Object::Integer(self.r as i64));
        dict.set(
            "O",
            Object::String(String::from_utf8_lossy(&self.o).to_string()),
        );
        dict.set(
            "U",
            Object::String(String::from_utf8_lossy(&self.u).to_string()),
        );
        dict.set("P", Object::Integer(self.p.bits() as i32 as i64));

        if !self.encrypt_metadata && self.v >= 4 {
            dict.set("EncryptMetadata", Object::Boolean(false));
        }

        // Add crypt filters if present
        if let Some(ref cf_list) = self.cf {
            let mut cf_dict = Dictionary::new();
            for filter in cf_list {
                cf_dict.set(&filter.name, Object::Dictionary(filter.to_dict()));
            }
            dict.set("CF", Object::Dictionary(cf_dict));
        }

        // Add stream filter
        if let Some(ref stm_f) = self.stm_f {
            match stm_f {
                StreamFilter::Identity => dict.set("StmF", Object::Name("Identity".to_string())),
                StreamFilter::StdCF => dict.set("StmF", Object::Name("StdCF".to_string())),
                StreamFilter::Custom(name) => dict.set("StmF", Object::Name(name.clone())),
            }
        }

        // Add string filter
        if let Some(ref str_f) = self.str_f {
            match str_f {
                StringFilter::Identity => dict.set("StrF", Object::Name("Identity".to_string())),
                StringFilter::StdCF => dict.set("StrF", Object::Name("StdCF".to_string())),
                StringFilter::Custom(name) => dict.set("StrF", Object::Name(name.clone())),
            }
        }

        dict
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypt_filter_method() {
        assert_eq!(CryptFilterMethod::None.pdf_name(), "None");
        assert_eq!(CryptFilterMethod::V2.pdf_name(), "V2");
        assert_eq!(CryptFilterMethod::AESV2.pdf_name(), "AESV2");
        assert_eq!(CryptFilterMethod::AESV3.pdf_name(), "AESV3");
    }

    #[test]
    fn test_crypt_filter() {
        let filter = CryptFilter::standard(CryptFilterMethod::V2);
        assert_eq!(filter.name, "StdCF");
        assert_eq!(filter.method, CryptFilterMethod::V2);
        assert_eq!(filter.length, Some(16));

        let dict = filter.to_dict();
        assert_eq!(dict.get("CFM"), Some(&Object::Name("V2".to_string())));
        assert_eq!(dict.get("Length"), Some(&Object::Integer(16)));
    }

    #[test]
    fn test_rc4_40bit_encryption_dict() {
        let owner_hash = vec![0u8; 32];
        let user_hash = vec![1u8; 32];
        let permissions = Permissions::new();

        let enc_dict = EncryptionDictionary::rc4_40bit(
            owner_hash.clone(),
            user_hash.clone(),
            permissions,
            None,
        );

        assert_eq!(enc_dict.filter, "Standard");
        assert_eq!(enc_dict.v, 1);
        assert_eq!(enc_dict.length, Some(5));
        assert_eq!(enc_dict.r, 2);
        assert_eq!(enc_dict.o, owner_hash);
        assert_eq!(enc_dict.u, user_hash);
    }

    #[test]
    fn test_rc4_128bit_encryption_dict() {
        let owner_hash = vec![0u8; 32];
        let user_hash = vec![1u8; 32];
        let permissions = Permissions::all();

        let enc_dict = EncryptionDictionary::rc4_128bit(
            owner_hash.clone(),
            user_hash.clone(),
            permissions,
            None,
        );

        assert_eq!(enc_dict.filter, "Standard");
        assert_eq!(enc_dict.v, 2);
        assert_eq!(enc_dict.length, Some(16));
        assert_eq!(enc_dict.r, 3);
    }

    #[test]
    fn test_encryption_dict_to_pdf() {
        let enc_dict =
            EncryptionDictionary::rc4_40bit(vec![0u8; 32], vec![1u8; 32], Permissions::new(), None);

        let pdf_dict = enc_dict.to_dict();
        assert_eq!(
            pdf_dict.get("Filter"),
            Some(&Object::Name("Standard".to_string()))
        );
        assert_eq!(pdf_dict.get("V"), Some(&Object::Integer(1)));
        assert_eq!(pdf_dict.get("Length"), Some(&Object::Integer(40))); // 5 bytes * 8 bits
        assert_eq!(pdf_dict.get("R"), Some(&Object::Integer(2)));
        assert!(pdf_dict.get("O").is_some());
        assert!(pdf_dict.get("U").is_some());
        assert!(pdf_dict.get("P").is_some());
    }
}
