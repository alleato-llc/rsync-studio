use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ItemizedChange {
    pub transfer_type: TransferType,
    pub file_type: FileType,
    pub differences: Vec<DifferenceKind>,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransferType {
    Sent,        // >
    Received,    // <
    LocalChange, // c
    NoUpdate,    // .
    Message,     // * (e.g., *deleting)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileType {
    File,      // f
    Directory, // d
    Symlink,   // L
    Device,    // D
    Special,   // S
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DifferenceKind {
    Checksum,           // c (position 2)
    Size,               // s (position 3)
    Timestamp,          // t (position 4)
    Permissions,        // p (position 5)
    Owner,              // o (position 6)
    Group,              // g (position 7)
    Acl,                // a (position 9)
    ExtendedAttributes, // x (position 10)
    NewlyCreated,       // + in any position
}
