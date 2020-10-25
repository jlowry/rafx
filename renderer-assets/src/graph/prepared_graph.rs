use super::{PhysicalImageId};
use fnv::{FnvHashMap, FnvHashSet};
use renderer_shell_vulkan::{VkDeviceContext, VkImage};
use ash::vk;
use crate::resources::ResourceLookupSet;
use crate::{ResourceArc, ImageViewResource, ResourceManager, ResourceContext};
use ash::prelude::VkResult;
use std::mem::ManuallyDrop;
use crate::vk_description as dsc;
use crate::vk_description::{ImageAspectFlags, SwapchainSurfaceInfo};
use crate::resources::RenderPassResource;
use crate::resources::FramebufferResource;
use crate::graph::graph_node::RenderGraphNodeId;
use ash::version::DeviceV1_0;
use crate::graph::{RenderGraphBuilder, RenderGraphImageUsageId};
use renderer_nodes::{RenderPhase, RenderPhaseIndex};
use crate::graph::graph_plan::RenderGraphPlan;

#[derive(Copy, Clone)]
pub struct RenderGraphContext<'a> {
    prepared_graph: &'a PreparedRenderGraph,
}

impl<'a> RenderGraphContext<'a> {
    pub fn image(
        &self,
        image: RenderGraphImageUsageId,
    ) -> Option<ResourceArc<ImageViewResource>> {
        self.prepared_graph.image(image)
    }

    pub fn device_context(&self) -> &VkDeviceContext {
        &self.prepared_graph.device_context
    }

    pub fn resource_context(&self) -> &ResourceContext {
        &self.prepared_graph.resource_context
    }
}

pub struct VisitRenderpassArgs<'a> {
    pub command_buffer: vk::CommandBuffer,
    pub renderpass: &'a ResourceArc<RenderPassResource>,
    pub graph_context: RenderGraphContext<'a>
}

/// Encapsulates a render graph plan and all resources required to execute it
pub struct PreparedRenderGraph {
    device_context: VkDeviceContext,
    resource_context: ResourceContext,
    image_resources: FnvHashMap<PhysicalImageId, ResourceArc<ImageViewResource>>,
    render_pass_resources: Vec<ResourceArc<RenderPassResource>>,
    framebuffer_resources: Vec<ResourceArc<FramebufferResource>>,
    graph_plan: RenderGraphPlan,
    swapchain_surface_info: SwapchainSurfaceInfo,
}

impl PreparedRenderGraph {
    pub fn new(
        device_context: &VkDeviceContext,
        resource_context: &ResourceContext,
        resources: &ResourceLookupSet,
        graph: RenderGraphBuilder,
        swapchain_surface_info: &SwapchainSurfaceInfo,
    ) -> VkResult<Self> {
        let graph_plan = graph.build_plan(swapchain_surface_info);

        let image_resources = Self::allocate_images(
            device_context,
            &graph_plan,
            resources,
            swapchain_surface_info,
        )?;
        let render_pass_resources =
            Self::allocate_render_passes(&graph_plan, resources, swapchain_surface_info)?;

        let framebuffer_resources = Self::allocate_framebuffers(
            &graph_plan,
            resources,
            swapchain_surface_info,
            &image_resources,
            &render_pass_resources,
        )?;

        Ok(PreparedRenderGraph {
            device_context: device_context.clone(),
            resource_context: resource_context.clone(),
            image_resources,
            render_pass_resources,
            framebuffer_resources,
            graph_plan,
            swapchain_surface_info: swapchain_surface_info.clone(),
        })
    }

    fn image(
        &self,
        image: RenderGraphImageUsageId,
    ) -> Option<ResourceArc<ImageViewResource>> {
        let physical_image = self.graph_plan.image_usage_to_physical.get(&image)?;
        self.image_resources.get(&physical_image).cloned()
    }

    fn allocate_images(
        device_context: &VkDeviceContext,
        graph: &RenderGraphPlan,
        resources: &ResourceLookupSet,
        swapchain_surface_info: &dsc::SwapchainSurfaceInfo,
    ) -> VkResult<FnvHashMap<PhysicalImageId, ResourceArc<ImageViewResource>>> {
        let mut image_resources: FnvHashMap<PhysicalImageId, ResourceArc<ImageViewResource>> =
            Default::default();

        for (id, image) in &graph.output_images {
            image_resources.insert(*id, image.dst_image.clone());
        }

        for (id, specification) in &graph.intermediate_images {
            let image = VkImage::new(
                device_context,
                vk_mem::MemoryUsage::GpuOnly,
                specification.usage_flags,
                vk::Extent3D {
                    width: swapchain_surface_info.extents.width,
                    height: swapchain_surface_info.extents.height,
                    depth: 1,
                },
                specification.format,
                vk::ImageTiling::OPTIMAL,
                specification.samples,
                1,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )?;
            let image_resource = resources.insert_image(ManuallyDrop::new(image));

            //println!("SPEC {:#?}", specification);
            let subresource_range = dsc::ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::from_bits(specification.aspect_flags.as_raw())
                    .unwrap(),
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            };

            let image_view_meta = dsc::ImageViewMeta {
                format: specification.format.into(),
                components: Default::default(),
                subresource_range,
                view_type: dsc::ImageViewType::Type2D,
            };
            let image_view =
                resources.get_or_create_image_view(&image_resource, &image_view_meta)?;

            image_resources.insert(*id, image_view);
        }
        Ok(image_resources)
    }

    fn allocate_render_passes(
        graph: &RenderGraphPlan,
        resources: &ResourceLookupSet,
        swapchain_surface_info: &dsc::SwapchainSurfaceInfo,
    ) -> VkResult<Vec<ResourceArc<RenderPassResource>>> {
        let mut render_pass_resources = Vec::with_capacity(graph.passes.len());
        for pass in &graph.passes {
            // println!("Allocate {:#?}", pass);
            // for dependency in &renderpass.description.dependencies {
            //     let builder = dependency.as_builder();
            //     let built = builder.build();
            //     println!("{:?}", built);
            // }
            let render_pass_resource =
                resources.get_or_create_renderpass(&pass.description, swapchain_surface_info)?;
            render_pass_resources.push(render_pass_resource);
        }
        Ok(render_pass_resources)
    }

    fn allocate_framebuffers(
        graph: &RenderGraphPlan,
        resources: &ResourceLookupSet,
        swapchain_surface_info: &dsc::SwapchainSurfaceInfo,
        image_resources: &FnvHashMap<PhysicalImageId, ResourceArc<ImageViewResource>>,
        render_pass_resources: &Vec<ResourceArc<RenderPassResource>>,
    ) -> VkResult<Vec<ResourceArc<FramebufferResource>>> {
        let mut framebuffers = Vec::with_capacity(graph.passes.len());
        for (pass_index, pass) in graph.passes.iter().enumerate() {
            let attachments: Vec<_> = pass
                .attachment_images
                .iter()
                .map(|x| image_resources[x].clone())
                .collect();

            let framebuffer_meta = dsc::FramebufferMeta {
                width: swapchain_surface_info.extents.width,
                height: swapchain_surface_info.extents.height,
                layers: 1,
            };

            let framebuffer = resources.get_or_create_framebuffer(
                render_pass_resources[pass_index].clone(),
                &attachments,
                &framebuffer_meta,
            )?;

            framebuffers.push(framebuffer);
        }
        Ok(framebuffers)
    }

    pub fn execute_graph(
        &self,
        node_visitor: &dyn RenderGraphNodeVisitor,
    ) -> VkResult<Vec<vk::CommandBuffer>> {
        //
        // Start a command writer. For now just do a single primary writer, later we can multithread this.
        //
        let mut command_writer = self
            .resource_context
            .dyn_command_writer_allocator()
            .allocate_writer(
                self.device_context
                    .queue_family_indices()
                    .graphics_queue_family_index,
                vk::CommandPoolCreateFlags::TRANSIENT,
                0,
            )?;

        let command_buffer = command_writer.begin_command_buffer(
            vk::CommandBufferLevel::PRIMARY,
            vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            None,
        )?;

        let device = self.device_context.device();

        let render_graph_context = RenderGraphContext {
            prepared_graph: &self,
        };

        //
        // Iterate through all passes
        //
        for (pass_index, pass) in self.graph_plan.passes.iter().enumerate() {
            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(self.render_pass_resources[pass_index].get_raw().renderpass)
                .framebuffer(self.framebuffer_resources[pass_index].get_raw().framebuffer)
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: self.swapchain_surface_info.extents,
                })
                .clear_values(&pass.clear_values);

            assert_eq!(pass.subpass_nodes.len(), 1);
            let node_id = pass.subpass_nodes[0];

            unsafe {

                if let Some(pre_pass_barrier) = &pass.pre_pass_barrier {
                    let mut image_memory_barriers = Vec::with_capacity(pre_pass_barrier.image_barriers.len());

                    for image_barrier in &pre_pass_barrier.image_barriers {
                        let image_view = &self.image_resources[&image_barrier.image];

                        let subresource_range = vk::ImageSubresourceRange::builder()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1);

                        let image_memory_barrier = vk::ImageMemoryBarrier::builder()
                            .src_access_mask(image_barrier.src_access)
                            .dst_access_mask(image_barrier.dst_access)
                            .old_layout(image_barrier.old_layout)
                            .new_layout(image_barrier.new_layout)
                            .src_queue_family_index(image_barrier.src_queue_family_index)
                            .dst_queue_family_index(image_barrier.dst_queue_family_index)
                            .image(image_view.get_raw().image.get_raw().image.image)
                            .subresource_range(*subresource_range)
                            .build();

                        image_memory_barriers.push(image_memory_barrier);
                    }

                    device.cmd_pipeline_barrier(
                        command_buffer,
                        pre_pass_barrier.src_stage,
                        pre_pass_barrier.dst_stage,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &image_memory_barriers
                    );
                }

                device.cmd_begin_render_pass(
                    command_buffer,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );

                let args = VisitRenderpassArgs {
                    renderpass: &self.render_pass_resources[pass_index],
                    graph_context: render_graph_context,
                    command_buffer
                };

                // callback here!
                node_visitor.visit_renderpass(node_id, args)?;

                device.cmd_end_render_pass(command_buffer);
            }
        }

        // for framebuffer in framebuffers {
        //     let device = self.device_context.device();
        //     use ash::version::DeviceV1_0;
        //     unsafe {
        //         device.destroy_framebuffer(framebuffer, None);
        //     }
        // }

        command_writer.end_command_buffer()?;

        Ok(vec![command_buffer])
    }
}

pub trait RenderGraphNodeVisitor {
    fn visit_renderpass(
        &self,
        node_id: RenderGraphNodeId,
        args: VisitRenderpassArgs
    ) -> VkResult<()>;
}

type RenderGraphNodeVisitorCallback<RenderGraphUserContextT> =
    dyn Fn(VisitRenderpassArgs, &RenderGraphUserContextT) -> VkResult<()> + Send;

/// Created by RenderGraphNodeCallbacks::create_visitor(). Implements RenderGraphNodeVisitor and
/// forwards the call, adding the user context as a parameter.
struct RenderGraphNodeVisitorImpl<'b, RenderGraphUserContextT> {
    context: &'b RenderGraphUserContextT,
    node_callbacks: &'b FnvHashMap<
        RenderGraphNodeId,
        Box<RenderGraphNodeVisitorCallback<RenderGraphUserContextT>>,
    >,
}

impl<'b, RenderGraphUserContextT> RenderGraphNodeVisitor
    for RenderGraphNodeVisitorImpl<'b, RenderGraphUserContextT>
{
    fn visit_renderpass(
        &self,
        node_id: RenderGraphNodeId,
        args: VisitRenderpassArgs
    ) -> VkResult<()> {
        (self.node_callbacks[&node_id])(args, self.context)
    }
}

/// All the callbacks associated with rendergraph nodes. We keep them separate from the nodes so
/// that we can avoid propagating generic parameters throughout the rest of the rendergraph code
pub struct RenderGraphNodeCallbacks<RenderGraphUserContextT> {
    node_callbacks:
        FnvHashMap<RenderGraphNodeId, Box<RenderGraphNodeVisitorCallback<RenderGraphUserContextT>>>,
    render_phase_dependencies: FnvHashMap<RenderGraphNodeId, FnvHashSet<RenderPhaseIndex>>,
}

impl<RenderGraphUserContextT> RenderGraphNodeCallbacks<RenderGraphUserContextT> {
    /// Adds a callback that receives the renderpass associated with the node
    pub fn set_renderpass_callback<CallbackFnT>(
        &mut self,
        node_id: RenderGraphNodeId,
        f: CallbackFnT,
    ) where
        CallbackFnT: Fn(
            VisitRenderpassArgs,
            &RenderGraphUserContextT
        ) -> VkResult<()>
            + 'static
            + Send,
    {
        self.node_callbacks.insert(node_id, Box::new(f));
    }

    pub fn add_renderphase_dependency<PhaseT: RenderPhase>(
        &mut self,
        node_id: RenderGraphNodeId,
    ) {
        self.render_phase_dependencies
            .entry(node_id)
            .or_default()
            .insert(PhaseT::render_phase_index());
    }

    /// Pass to PreparedRenderGraph::execute_graph, this will cause the graph to be executed,
    /// triggering any registered callbacks
    pub fn create_visitor<'a>(
        &'a self,
        context: &'a RenderGraphUserContextT,
    ) -> Box<dyn RenderGraphNodeVisitor + 'a> {
        Box::new(RenderGraphNodeVisitorImpl::<'a, RenderGraphUserContextT> {
            context,
            node_callbacks: &self.node_callbacks,
        })
    }
}

impl<T> Default for RenderGraphNodeCallbacks<T> {
    fn default() -> Self {
        RenderGraphNodeCallbacks {
            node_callbacks: Default::default(),
            render_phase_dependencies: Default::default(),
        }
    }
}

/// A wrapper around a prepared render graph and callbacks that will be hit when executing the graph
pub struct RenderGraphExecutor<T> {
    prepared_graph: PreparedRenderGraph,
    callbacks: RenderGraphNodeCallbacks<T>,
}

impl<T> RenderGraphExecutor<T> {
    /// Create the executor. This allows the prepared graph, resources required to execute it, and
    /// callbacks that will be triggered while executing it to be passed around and executed later.
    pub fn new(
        device_context: &VkDeviceContext,
        resource_context: &ResourceContext,
        resource_manager: &mut ResourceManager,
        graph: RenderGraphBuilder,
        swapchain_surface_info: &SwapchainSurfaceInfo,
        callbacks: RenderGraphNodeCallbacks<T>,
    ) -> VkResult<Self> {
        //
        // Allocate the resources for the graph
        //
        let prepared_graph = PreparedRenderGraph::new(
            device_context,
            resource_context,
            resource_context.resources(),
            graph,
            swapchain_surface_info,
        )?;

        //
        // Ensure expensive resources are persisted across frames so they can be reused
        //
        //TODO: Make resource caches MT-friendly
        for renderpass in &prepared_graph.render_pass_resources {
            resource_manager
                .resource_caches_mut()
                .cache_render_pass(renderpass.clone());
        }

        for framebuffer in &prepared_graph.framebuffer_resources {
            resource_manager
                .resource_caches_mut()
                .cache_framebuffer(framebuffer.clone());
        }

        //
        // Pre-warm caches for pipelines that we may need
        //

        for (node_id, render_phase_indices) in &callbacks.render_phase_dependencies {
            // Passes may get culled if the images are not used. This means the renderpass would
            // not be created so pipelines are also not needed
            if let Some(&renderpass_index) = prepared_graph
                .graph_plan
                .node_to_renderpass_index
                .get(node_id)
            {
                for &render_phase_index in render_phase_indices {
                    resource_manager
                        .graphics_pipeline_cache()
                        .register_renderpass_to_phase_index_per_frame(
                            &prepared_graph.render_pass_resources[renderpass_index],
                            render_phase_index,
                        )
                }
            }
        }
        resource_manager
            .graphics_pipeline_cache()
            .precache_pipelines_for_all_phases()?;

        //
        // Return the executor which can be triggered later
        //
        Ok(RenderGraphExecutor {
            prepared_graph,
            callbacks,
        })
    }

    pub fn renderpass_resource(
        &self,
        node_id: RenderGraphNodeId,
    ) -> Option<ResourceArc<RenderPassResource>> {
        let renderpass_index = *self
            .prepared_graph
            .graph_plan
            .node_to_renderpass_index
            .get(&node_id)?;
        Some(self.prepared_graph.render_pass_resources[renderpass_index].clone())
    }

    pub fn image_resource(
        &self,
        image_usage: RenderGraphImageUsageId,
    ) -> Option<ResourceArc<ImageViewResource>> {
        let image = self
            .prepared_graph
            .graph_plan
            .image_usage_to_physical
            .get(&image_usage)?;
        Some(self.prepared_graph.image_resources[image].clone())
    }

    /// Executes the graph, passing through the given context parameter
    pub fn execute_graph(
        self,
        context: &T,
    ) -> VkResult<Vec<vk::CommandBuffer>> {
        let visitor = self.callbacks.create_visitor(context);
        self.prepared_graph.execute_graph(&*visitor)
    }
}
