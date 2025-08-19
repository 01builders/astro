//! Generated protobuf types for CometBFT
//!
//! This crate contains auto-generated Rust types from the CometBFT protobuf definitions.

#![allow(clippy::all)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(unused_variables)]

// Include the generated protobuf code from codegen directory
mod codegen;

// Re-export the CometBFT types for convenient access
pub use codegen::cometbft;

#[cfg(test)]
mod tests {
    #[test]
    fn it_builds() {}
}