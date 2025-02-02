use rafx::render_feature_renderer_prelude::*;

use super::{SpriteExtractJob, SpriteRenderFeature, SpriteRenderNodeSet};
use distill::loader::handle::Handle;
use rafx::assets::MaterialAsset;

pub struct SpriteStaticResources {
    pub sprite_material: Handle<MaterialAsset>,
}

pub struct SpriteRendererPlugin;

impl SpriteRendererPlugin {
    pub fn legion_init(resources: &mut legion::Resources) {
        resources.insert(SpriteRenderNodeSet::default());
    }

    pub fn legion_destroy(resources: &mut legion::Resources) {
        resources.remove::<SpriteRenderNodeSet>();
    }
}

impl RendererPlugin for SpriteRendererPlugin {
    fn configure_render_registry(
        &self,
        render_registry: RenderRegistryBuilder,
    ) -> RenderRegistryBuilder {
        render_registry.register_feature::<SpriteRenderFeature>()
    }

    fn initialize_static_resources(
        &self,
        asset_manager: &mut AssetManager,
        asset_resource: &mut AssetResource,
        _extract_resources: &ExtractResources,
        render_resources: &mut ResourceMap,
        _upload: &mut RafxTransferUpload,
    ) -> RafxResult<()> {
        let sprite_material =
            asset_resource.load_asset_path::<MaterialAsset, _>("materials/sprite.material");

        asset_manager.wait_for_asset_to_load(
            &sprite_material,
            asset_resource,
            "sprite_material",
        )?;

        render_resources.insert(SpriteStaticResources { sprite_material });

        Ok(())
    }

    fn add_extract_jobs(
        &self,
        _extract_resources: &ExtractResources,
        _render_resources: &RenderResources,
        extract_jobs: &mut Vec<Box<dyn ExtractJob>>,
    ) {
        extract_jobs.push(Box::new(SpriteExtractJob::new()));
    }
}
