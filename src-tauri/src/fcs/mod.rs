//! Flow cytometry standard (.FCS) file manipulation operations.
//!
//! This module contains basic methods to manipulate the contents of .fcs files.

pub mod byteorder;
pub mod cache;
pub mod cached_fcs;
pub mod datatype;
pub mod file;
pub mod filter_criteria;
pub mod header;
pub mod keyword;
pub mod metadata;
pub mod parameter;
pub mod robust;
pub mod spillover_commands;
pub mod spillover_groups;
pub mod version;

use self::parameter::{
    ChannelName, Parameter, ParameterCategory, ParameterOption, ParameterProcessing,
};
use rustc_hash::FxHashMap;
use std::path::PathBuf;

pub type GUID = String;
pub type FileKeyword = String;
pub type FilePath = PathBuf;
pub type ParameterCount = usize;
pub type EventCount = usize;
pub type ParameterMap = FxHashMap<ChannelName, Parameter>;
