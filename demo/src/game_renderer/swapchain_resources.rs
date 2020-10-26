use renderer::vulkan::{VkDeviceContext, VkSwapchain, SwapchainInfo};
use crate::game_renderer::GameRendererInner;
use renderer::assets::resources::{
    ResourceManager, DynDescriptorSet, ResourceArc, ImageViewResource, RenderPassResource,
};
use renderer::assets::RenderpassAsset;
use renderer::assets::vk_description::SwapchainSurfaceInfo;
use ash::prelude::VkResult;
use renderer::assets::vk_description as dsc;
use renderer::vulkan::VkImageRaw;
use atelier_assets::loader::handle::Handle;

pub struct SwapchainResources {
    // The images presented by the swapchain
    //TODO: We don't properly support multiple swapchains right now. This would ideally be a map
    // of window/surface to info for the swapchain
    pub swapchain_images: Vec<ResourceArc<ImageViewResource>>,

    pub debug_material_per_frame_data: DynDescriptorSet,

    pub swapchain_info: SwapchainInfo,
    pub swapchain_surface_info: SwapchainSurfaceInfo,
}

impl SwapchainResources {
    pub fn new(
        _device_context: &VkDeviceContext,
        swapchain: &VkSwapchain,
        game_renderer: &mut GameRendererInner,
        resource_manager: &mut ResourceManager,
        swapchain_info: SwapchainInfo,
        swapchain_surface_info: SwapchainSurfaceInfo,
    ) -> VkResult<SwapchainResources> {
        log::debug!("creating swapchain resources");

        //
        // Create resources for the swapchain images. This allows renderer systems to use them
        // interchangably with non-swapchain images
        //
        let image_view_meta = dsc::ImageViewMeta {
            view_type: dsc::ImageViewType::Type2D,
            subresource_range: dsc::ImageSubresourceRange {
                aspect_mask: dsc::ImageAspectFlag::Color.into(),
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            components: dsc::ComponentMapping::default(),
            format: swapchain.swapchain_info.surface_format.format.into(),
        };

        let mut swapchain_images = Vec::with_capacity(swapchain.swapchain_images.len());
        for &image in &swapchain.swapchain_images {
            let raw = VkImageRaw {
                allocation: None,
                image,
            };

            let image = resource_manager.resources().insert_raw_image(raw);
            let image_view = resource_manager
                .resources()
                .get_or_create_image_view(&image, &image_view_meta)?;

            swapchain_images.push(image_view);
        }

        let mut descriptor_set_allocator = resource_manager.create_descriptor_set_allocator();

        let debug_per_frame_layout = resource_manager
            .get_descriptor_set_layout_for_pass(
                &game_renderer.static_resources.debug3d_material,
                0,
                0,
            )
            .unwrap();
        let debug_material_per_frame_data = descriptor_set_allocator
            .create_dyn_descriptor_set_uninitialized(&debug_per_frame_layout)?;

        log::debug!("game renderer swapchain_created finished");

        VkResult::Ok(SwapchainResources {
            swapchain_images,
            debug_material_per_frame_data,
            swapchain_info,
            swapchain_surface_info,
        })
    }

    fn create_renderpass_resource(
        resource_manager: &mut ResourceManager,
        swapchain_surface_info: &SwapchainSurfaceInfo,
        asset_handle: &Handle<RenderpassAsset>,
    ) -> VkResult<ResourceArc<RenderPassResource>> {
        use atelier_assets::loader::handle::AssetHandle;
        let renderpass_asset_data = resource_manager
            .loaded_assets()
            .renderpasses
            .get_committed(asset_handle.load_handle())
            .unwrap()
            .data
            .clone();

        resource_manager
            .resources()
            .get_or_create_renderpass(&renderpass_asset_data.renderpass, swapchain_surface_info)
    }
}
