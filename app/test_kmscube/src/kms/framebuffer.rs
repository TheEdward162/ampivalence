use anyhow::Context;

use drm::{
	control::{framebuffer::Handle as FramebufferHandle, Mode, Device},
	buffer::{DrmFourcc, DrmModifier}
};
use gbm::{BufferObject, BufferObjectFlags};

use super::KmsDevice;

pub struct FrameBufferObject {
	device: KmsDevice,
	buffer: BufferObject<()>,
	framebuffer: FramebufferHandle
}
impl FrameBufferObject {
	pub fn new(
		device: KmsDevice,
		mode: &Mode,
		format: DrmFourcc,
		modifier: DrmModifier
	) -> anyhow::Result<Self> {
		log::trace!("Creating buffer object with {:?} {:?} {:?}", modifier, format, mode.size());
		// let buffer = device.create_buffer_object_with_modifiers(
		// 	mode.size().0 as u32, mode.size().1 as u32,
		// 	format, std::iter::once(modifier)
		// ).context("Failed to create buffer object")?;

		let buffer = device.create_buffer_object(
			mode.size().0 as u32, mode.size().1 as u32,
			format, BufferObjectFlags::RENDERING | BufferObjectFlags::SCANOUT
		).context("Failed to create buffer object")?;

		let framebuffer = device.add_planar_framebuffer(
			&buffer,
			&[Some(modifier), None, None, None],
			0
		).context("Failed to create framebuffer")?;
		
		Ok(
			FrameBufferObject {
				device,
				buffer,
				framebuffer
			}
		)
	}

	pub fn buffer(&self) -> &BufferObject<()> {
		&self.buffer
	}

	pub fn framebuffer(&self) -> FramebufferHandle {
		self.framebuffer
	}
}
impl Drop for FrameBufferObject {
    fn drop(&mut self) {
        self.device.destroy_framebuffer(self.framebuffer).expect("Failed to destroy framebuffer");
    }
}
