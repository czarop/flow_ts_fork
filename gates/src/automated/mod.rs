//! Automated gating algorithms
//!
//! This module provides automated gate generation for common flow cytometry
//! preprocessing steps, including scatter gating and doublet detection.

pub mod scatter;
pub mod doublets;
pub mod interactive;
pub mod comparison;

pub use scatter::{ScatterGateConfig, ScatterGateMethod, ScatterGateResult, create_scatter_gate, ClusterAlgorithm};
pub use doublets::{DoubletGateConfig, DoubletMethod, DoubletGateResult, detect_doublets};
pub use interactive::{UserReview, PipelineBreakpoint};
pub use comparison::{compare_doublet_methods, compare_with_peacoqc, DoubletComparisonResult, MethodResult};

use crate::{Gate, GateError};
use crate::hierarchy::GateHierarchy;
use flow_fcs::Fcs;

/// Configuration for preprocessing pipeline
#[derive(Debug, Clone)]
pub struct PreprocessingConfig {
    /// Scatter gate configuration
    pub scatter_config: ScatterGateConfig,
    /// Doublet detection configuration
    pub doublet_config: DoubletGateConfig,
}

/// Result of preprocessing pipeline
#[derive(Debug)]
pub struct PreprocessingGates {
    /// Scatter gate
    pub scatter_gate: Option<Gate>,
    /// Doublet exclusion gate (if generated)
    pub doublet_gate: Option<Gate>,
    /// Gate hierarchy
    pub hierarchy: GateHierarchy,
}

/// Fully automated preprocessing pipeline
///
/// Creates scatter gate and doublet exclusion gate automatically.
pub fn create_preprocessing_gates(
    fcs: &Fcs,
    config: PreprocessingConfig,
) -> Result<PreprocessingGates, crate::GateError> {
    let mut hierarchy = GateHierarchy::new();

    // 1. Scatter gate (multi-population)
    let scatter_result = create_scatter_gate(fcs, &config.scatter_config)?;
    // Note: Gates are stored separately, hierarchy tracks relationships
    // If scatter gate has a parent, we'd add it here: hierarchy.add_child(parent_id, gate.id())

    // 2. Doublet exclusion
    let doublet_result = detect_doublets(fcs, &config.doublet_config)?;
    // If doublet gate should be a child of scatter gate, add relationship:
    // if let (Some(scatter_gate), Some(doublet_gate)) = (&scatter_result.gate, &doublet_result.exclusion_gate) {
    //     hierarchy.add_child(scatter_gate.id(), doublet_gate.id());
    // }

    Ok(PreprocessingGates {
        scatter_gate: scatter_result.gate,
        doublet_gate: doublet_result.exclusion_gate,
        hierarchy,
    })
}

/// Semi-automated preprocessing pipeline with user review breakpoints
///
/// Allows user to review and tweak gates at each step before proceeding.
pub fn create_preprocessing_gates_interactive(
    fcs: &Fcs,
    config: PreprocessingConfig,
    review_callback: impl Fn(PipelineBreakpoint) -> UserReview,
) -> Result<PreprocessingGates, crate::GateError> {
    let mut hierarchy = GateHierarchy::new();

    // 1. Scatter gate (with user review)
    let scatter_result = create_scatter_gate(fcs, &config.scatter_config)?;
    let scatter_review = review_callback(PipelineBreakpoint::ScatterGate(scatter_result.clone()));
    
    if let UserReview::Accept = scatter_review {
        // Gate stored in result, hierarchy tracks relationships if needed
    }

    // 2. Doublet exclusion (with user review)
    let doublet_result = detect_doublets(fcs, &config.doublet_config)?;
    let doublet_review = review_callback(PipelineBreakpoint::DoubletGate(doublet_result.clone()));
    
    if let UserReview::Accept = doublet_review {
        // Gate stored in result, hierarchy tracks relationships if needed
        // If doublet should be child of scatter:
        // if let (Some(scatter_gate), Some(doublet_gate)) = (&scatter_result.gate, &doublet_result.exclusion_gate) {
        //     hierarchy.add_child(scatter_gate.id(), doublet_gate.id());
        // }
    }

    Ok(PreprocessingGates {
        scatter_gate: scatter_result.gate,
        doublet_gate: doublet_result.exclusion_gate,
        hierarchy,
    })
}
