use rafx::nodes::RenderPhase;
use rafx::nodes::{RenderPhaseIndex, SubmitNode};

rafx::declare_render_phase!(
    TransparentRenderPhase,
    TRANSPARENT_RENDER_PHASE_INDEX,
    transparent_render_phase_sort_submit_nodes
);

#[profiling::function]
fn transparent_render_phase_sort_submit_nodes(
    mut submit_nodes: Vec<SubmitNode>
) -> Vec<SubmitNode> {
    // Sort by distance from camera back to front
    log::trace!(
        "Sort phase {}",
        TransparentRenderPhase::render_phase_debug_name()
    );
    submit_nodes.sort_unstable_by(|a, b| b.distance().partial_cmp(&a.distance()).unwrap());

    submit_nodes
}
