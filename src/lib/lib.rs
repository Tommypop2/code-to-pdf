#![warn(missing_docs)]

//! # Code To PDF
//!
//! This crate provides primitives for generating PDFs containing syntax-highlighted code
//!
//! [`code_to_pdf::CodeToPdf`] is the main struct for this so is likely the best place to start

pub mod code_to_pdf;
pub mod dimensions;
pub mod font_loader;
pub mod helpers;
pub mod text_manipulation;
mod logging;