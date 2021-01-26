// This code is auto-generated by the shader processor.

#[allow(unused_imports)]
use rafx_framework::RafxResult;

#[allow(unused_imports)]
use rafx_framework::{
    DescriptorSetAllocator, DescriptorSetArc, DescriptorSetInitializer, DynDescriptorSet,
    ImageViewResource, ResourceArc,
};

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct ArgsStd140 {
    pub mvp: [[f32; 4]; 4], // +0 (size: 64)
} // 64 bytes

impl Default for ArgsStd140 {
    fn default() -> Self {
        ArgsStd140 {
            mvp: <[[f32; 4]; 4]>::default(),
        }
    }
}

pub type ArgsUniform = ArgsStd140;

pub const UNIFORM_BUFFER_DESCRIPTOR_SET_INDEX: usize = 0;
pub const UNIFORM_BUFFER_DESCRIPTOR_BINDING_INDEX: usize = 0;
pub const SMP_DESCRIPTOR_SET_INDEX: usize = 0;
pub const SMP_DESCRIPTOR_BINDING_INDEX: usize = 1;
pub const TEX_DESCRIPTOR_SET_INDEX: usize = 1;
pub const TEX_DESCRIPTOR_BINDING_INDEX: usize = 0;

pub struct DescriptorSet0Args<'a> {
    pub uniform_buffer: &'a ArgsUniform,
}

impl<'a> DescriptorSetInitializer<'a> for DescriptorSet0Args<'a> {
    type Output = DescriptorSet0;

    fn create_dyn_descriptor_set(
        descriptor_set: DynDescriptorSet,
        args: Self,
    ) -> Self::Output {
        let mut descriptor = DescriptorSet0(descriptor_set);
        descriptor.set_args(args);
        descriptor
    }

    fn create_descriptor_set(
        descriptor_set_allocator: &mut DescriptorSetAllocator,
        descriptor_set: DynDescriptorSet,
        args: Self,
    ) -> RafxResult<DescriptorSetArc> {
        let mut descriptor = Self::create_dyn_descriptor_set(descriptor_set, args);
        descriptor.0.flush(descriptor_set_allocator)?;
        Ok(descriptor.0.descriptor_set().clone())
    }
}

pub struct DescriptorSet0(pub DynDescriptorSet);

impl DescriptorSet0 {
    pub fn set_args_static(
        descriptor_set: &mut DynDescriptorSet,
        args: DescriptorSet0Args,
    ) {
        descriptor_set.set_buffer_data(
            UNIFORM_BUFFER_DESCRIPTOR_BINDING_INDEX as u32,
            args.uniform_buffer,
        );
    }

    pub fn set_args(
        &mut self,
        args: DescriptorSet0Args,
    ) {
        self.set_uniform_buffer(args.uniform_buffer);
    }

    pub fn set_uniform_buffer(
        &mut self,
        uniform_buffer: &ArgsUniform,
    ) {
        self.0.set_buffer_data(
            UNIFORM_BUFFER_DESCRIPTOR_BINDING_INDEX as u32,
            uniform_buffer,
        );
    }

    pub fn flush(
        &mut self,
        descriptor_set_allocator: &mut DescriptorSetAllocator,
    ) -> RafxResult<()> {
        self.0.flush(descriptor_set_allocator)
    }
}

pub struct DescriptorSet1Args<'a> {
    pub tex: &'a ResourceArc<ImageViewResource>,
}

impl<'a> DescriptorSetInitializer<'a> for DescriptorSet1Args<'a> {
    type Output = DescriptorSet1;

    fn create_dyn_descriptor_set(
        descriptor_set: DynDescriptorSet,
        args: Self,
    ) -> Self::Output {
        let mut descriptor = DescriptorSet1(descriptor_set);
        descriptor.set_args(args);
        descriptor
    }

    fn create_descriptor_set(
        descriptor_set_allocator: &mut DescriptorSetAllocator,
        descriptor_set: DynDescriptorSet,
        args: Self,
    ) -> RafxResult<DescriptorSetArc> {
        let mut descriptor = Self::create_dyn_descriptor_set(descriptor_set, args);
        descriptor.0.flush(descriptor_set_allocator)?;
        Ok(descriptor.0.descriptor_set().clone())
    }
}

pub struct DescriptorSet1(pub DynDescriptorSet);

impl DescriptorSet1 {
    pub fn set_args_static(
        descriptor_set: &mut DynDescriptorSet,
        args: DescriptorSet1Args,
    ) {
        descriptor_set.set_image(TEX_DESCRIPTOR_BINDING_INDEX as u32, args.tex);
    }

    pub fn set_args(
        &mut self,
        args: DescriptorSet1Args,
    ) {
        self.set_tex(args.tex);
    }

    pub fn set_tex(
        &mut self,
        tex: &ResourceArc<ImageViewResource>,
    ) {
        self.0.set_image(TEX_DESCRIPTOR_BINDING_INDEX as u32, tex);
    }

    pub fn flush(
        &mut self,
        descriptor_set_allocator: &mut DescriptorSetAllocator,
    ) -> RafxResult<()> {
        self.0.flush(descriptor_set_allocator)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_struct_args_std140() {
        assert_eq!(std::mem::size_of::<ArgsStd140>(), 64);
        assert_eq!(std::mem::size_of::<[[f32; 4]; 4]>(), 64);
        assert_eq!(std::mem::align_of::<[[f32; 4]; 4]>(), 4);
        assert_eq!(memoffset::offset_of!(ArgsStd140, mvp), 0);
    }
}
