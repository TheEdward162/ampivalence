use anyhow::Context as AnyhowContext;

use gbm::{AsRaw, Format};

use khronos_egl as egl;
use egl::{
	DynamicInstance,
	Display,
	Context
};

use crate::kms::{KmsContext, FrameBufferObject};

pub struct EglContext {
	format: Format,
	instance: DynamicInstance<egl::EGL1_5>,
	display: Display,
	context: Context
}
impl EglContext {
	pub fn new(
		kms: &KmsContext,
		format: Format
	) -> anyhow::Result<Self> {
		// SAFETY: *eyeroll*
		let instance = unsafe {
			egl::DynamicInstance::<egl::EGL1_5>::load_required().context("Failed to load libEGL.so.1")?
		};

		// #define EGL_PLATFORM_GBM_KHR              0x31D7
		let display = instance.get_platform_display(
			0x31D7, kms.device().as_raw() as *mut _, &[egl::ATTRIB_NONE]
		).context("Failed to get platform display")?;

		instance.initialize(display).context("Failed to initialize EGL")?;

		instance.bind_api(egl::OPENGL_ES_API).context("Failed to bind EGL API")?;

		let mut configs = Vec::with_capacity(
			instance.get_config_count(display).context("Failed to get config count")?
		);
		instance.choose_config(
			display,
			&[
				egl::SURFACE_TYPE, egl::WINDOW_BIT,
				egl::RED_SIZE, 1,
				egl::GREEN_SIZE, 1,
				egl::BLUE_SIZE, 1,
				egl::ALPHA_SIZE, 0,
				egl::RENDERABLE_TYPE, egl::OPENGL_ES2_BIT,
				egl::SAMPLES, 0,
				egl::NONE
			],
			&mut configs
		).context("Failed to get configs")?;

		let mut chosen_config = None;
		for config in configs {
			let visual_id = instance.get_config_attrib(display, config, egl::NATIVE_VISUAL_ID).context("Failed to get config visual id")?;

			if visual_id as u32 == format as u32 {
				chosen_config = Some(config);
				break;
			}
		}
		let chosen_config = chosen_config.context("Failed to choose a config")?;

		let context = instance.create_context(
			display,
			chosen_config,
			None,
			&[
				egl::CONTEXT_CLIENT_VERSION, 2,
				egl::NONE
			]
		).context("Failed to create EGL contex")?;

		instance.make_current(display, None, None, Some(context)).context("Failed to bind EGL current context")?;

		Ok(
			EglContext {
				format,
				instance,
				display,
				context
			}
		)
	}

	fn create_framebuffer(&self, fbo: &FrameBufferObject) -> anyhow::Result<()> {
		let fd = fbo.buffer().fd().context("Failed to get buffer object DMA fd")?;

		// #define EGL_LINUX_DMA_BUF_EXT          0x3270
		// #define EGL_LINUX_DRM_FOURCC_EXT        0x3271
		// #define EGL_DMA_BUF_PLANE0_FD_EXT       0x3272
        // #define EGL_DMA_BUF_PLANE0_OFFSET_EXT   0x3273
        // #define EGL_DMA_BUF_PLANE0_PITCH_EXT    0x3274
		let image = self.instance.create_image(
			self.display,
			self.context,
			0x3270,
			unsafe { egl::ClientBuffer::from_ptr(std::ptr::null_mut()) },
			&[
				egl::WIDTH as _, fbo.buffer().width().unwrap() as _,
				egl::HEIGHT as _, fbo.buffer().height().unwrap() as _,
				0x3271, fbo.buffer().format().unwrap() as _,
				0x3272, fd as _,
				0x3273, 0,
				0x3274, fbo.buffer().stride().unwrap() as _,
				egl::ATTRIB_NONE
			]
		).context("Failed to create EGL image")?;

		// TODO: Close fd?
		
		todo!()
	}
}
