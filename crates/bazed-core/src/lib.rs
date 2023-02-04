#![forbid(unreachable_pub)]
#![allow(rustdoc::private_intra_doc_links)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod app;
pub mod buffer;
pub mod document;
pub mod highlighting;
pub mod region;
mod user_buffer_op;
pub mod view;
mod vim_interface;
mod word_boundary;

#[cfg(test)]
mod test_util;
