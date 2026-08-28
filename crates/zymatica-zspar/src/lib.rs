#![forbid(unsafe_code)]

pub mod crc32c;
pub mod gf16;
pub mod rs12_8;
pub mod semantic;
pub mod sha256;

pub use rs12_8::{DecodeResult, DecodeStatus, Rs12_8};
pub use semantic::{
    axis_diff_mask, semantic_tag, stable_text_id, Concept8D, InvariantKind, InvariantPatchFrame,
    InvariantRecord, InvariantSet, ParityOnlyFrame, RepairRequest, RepairResponse, RepairResult,
    RepairStatus, SystematicFrame,
};
