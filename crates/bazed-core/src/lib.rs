#![forbid(unreachable_pub)]
#![allow(rustdoc::private_intra_doc_links)]

pub mod app;
pub mod buffer;
pub mod document;
mod input_mapper;
pub mod region;
mod user_buffer_op;
pub mod view;
mod word_boundary;

#[cfg(test)]
mod test_util;
