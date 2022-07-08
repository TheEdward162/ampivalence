use std::path::Path;

use anyhow::Context;

use drm::{
	Device,
	ClientCapability,
	control::{
		Device as ControlDevice,
		Mode,
		connector::{Info as ConnectorInfo},
		plane::{Info as PlaneInfo},
		property::{Handle as PropertyHandle, Value as PropertyValue}
	},
	buffer::{DrmFourcc, DrmModifier}
};
use gbm::Device as GbmDevice;

type KmsDevice = GbmDevice<DrmDevice>;

mod device;
mod framebuffer;

use device::{DrmDevice, IndexedCrtc};
pub use framebuffer::FrameBufferObject;

struct CommitPropertyCache {
	/// connector property `CRTC_ID`
	pub connector_crtc_id: PropertyHandle,
	/// crtc property `MODE_ID`
	pub crtc_mode_id: PropertyHandle,
	/// crtc property `ACTIVE`
	pub crtc_active: PropertyHandle,
	// /// crtc property `OUT_FENCE_PTR`
	// pub crtc_out_fence_ptr: PropertyHandle,
	/// plane property `IN_FENCE_FD`
	pub plane_in_fence_fd: PropertyHandle,
	/// plane property `FB_ID`
	pub plane_fb_id: PropertyHandle,
	/// plane property `CRTC_ID`
	pub plane_crtc_id: PropertyHandle,
	/// plane property `SRC_X`
	pub plane_src_x: PropertyHandle,
	/// plane property `SRC_Y`
	pub plane_src_y: PropertyHandle,
	/// plane property `SRC_W`
	pub plane_src_w: PropertyHandle,
	/// plane property `SRC_H`
	pub plane_src_h: PropertyHandle,
	/// plane property `CRTC_X`
	pub plane_crtc_x: PropertyHandle,
	/// plane property `CRTC_Y`
	pub plane_crtc_y: PropertyHandle,
	/// plane property `CRTC_W`
	pub plane_crtc_w: PropertyHandle,
	/// plane property `CRTC_H`
	pub plane_crtc_h: PropertyHandle,
	/// blob containing mode
	pub blob_mode: PropertyValue<'static>
}

pub struct KmsContext {
	device: KmsDevice,
	connector: ConnectorInfo,
	mode: Mode,
	crtc: IndexedCrtc,
	plane: PlaneInfo,
	property_cache: CommitPropertyCache
}
impl KmsContext {
	fn cache_commit_properties(
		device: &KmsDevice,
		connector: &ConnectorInfo,
		crtc: &IndexedCrtc,
		plane: &PlaneInfo,
		mode: &Mode
	) -> anyhow::Result<CommitPropertyCache> {
		macro_rules! find_properties {
			(
				$handles: expr;
				$( let $var_name: ident = $prop_name: literal; )+
			) => {
				$(
					let mut $var_name: Option<PropertyHandle> = None;
				)+
				for &handle in $handles {
					let property = device.get_property(handle).context("Failed to query property")?;

					match property.name().to_str() {
						$(
							Ok($prop_name) => {
								assert!($var_name.is_none());
								$var_name = Some(handle);
							}
						)+
						_ => ()
					}
				}
				$(
					let $var_name: PropertyHandle = $var_name.context(concat!("Could not find property ", $prop_name))?;
				)+
			}
		}

		let connector_properties = device.get_properties(connector.handle()).context("Failed to query connector properties")?;
		find_properties!(
			connector_properties.as_props_and_values().0;
			let connector_crtc_id = "CRTC_ID";
		);

		let crtc_properties = device.get_properties(crtc.info.handle()).context("Failed to query crtc properties")?;
		find_properties!(
			crtc_properties.as_props_and_values().0;
			let crtc_mode_id = "MODE_ID";
			let crtc_active = "ACTIVE";
			// let crtc_out_fence_ptr = "OUT_FENCE_PTR";
		);

		let plane_properties = device.get_properties(plane.handle()).context("Failed to query plane properties")?;
		find_properties!(
			plane_properties.as_props_and_values().0;
			let plane_in_fence_fd = "IN_FENCE_FD";
			let plane_fb_id = "FB_ID";
			let plane_crtc_id = "CRTC_ID";
			let plane_src_x = "SRC_X";
			let plane_src_y = "SRC_Y";
			let plane_src_w = "SRC_W";
			let plane_src_h = "SRC_H";
			let plane_crtc_x = "CRTC_X";
			let plane_crtc_y = "CRTC_Y";
			let plane_crtc_w = "CRTC_W";
			let plane_crtc_h = "CRTC_H";
		);

		Ok(
			CommitPropertyCache {
				connector_crtc_id,
				crtc_mode_id,
				crtc_active,
				// crtc_out_fence_ptr,
				plane_in_fence_fd,
				plane_fb_id,
				plane_crtc_id,
				plane_src_x,
				plane_src_y,
				plane_src_w,
				plane_src_h,
				plane_crtc_x,
				plane_crtc_y,
				plane_crtc_w,
				plane_crtc_h,
				blob_mode: device.create_property_blob(mode).context("Failed to crate property blob")?
			}
		)
	}
	
	pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
		let device = DrmDevice::new(path).context("Failed to open drm device")?;

		let resource_handles = device.resource_handles().context("Failed to query control device resources")?;
		let connector = device.choose_connector(resource_handles.connectors())?;
		let mode = device.choose_mode(connector.modes())?;
		let crtc = device.choose_crtc(
			connector.current_encoder(),
			connector.encoders(),
			resource_handles.crtcs()
		)?;

		device.set_client_capability(ClientCapability::Atomic, true).context("Failed to set Atomic client capability")?;
		// device.set_client_capability(ClientCapability::UniversalPlanes, true).context("Failed to set UniversalPlanes capability")?;
		let plane = device.choose_plane(&crtc)?;

		let device = GbmDevice::new(device).context("Failed to create gbm device")?;

		let property_cache = Self::cache_commit_properties(&device, &connector, &crtc, &plane, &mode).context("Failed to cache commit properties")?;

		/*
		let framebuffers: [FrameBufferObject; FRAMEBUFFERS] = {
			let mut framebuffers = Vec::with_capacity(FRAMEBUFFERS);
			for _ in 0 .. FRAMEBUFFERS {
				let fbo = FrameBufferObject::new(&device, &mode, format, modifier)?;
				framebuffers.push(fbo);
			}
			
			let mut it = framebuffers.into_iter();
			
			[(); FRAMEBUFFERS].map(|_| it.next().unwrap())
		};
		*/

		/*
		let framebuffers: [FrameBufferObject; FRAMEBUFFERS] = {
			use std::mem::MaybeUninit;

			let mut mem = MaybeUninit::<[FrameBufferObject; FRAMEBUFFERS]>::uninit();

			for i in 0 .. FRAMEBUFFERS {
				match FrameBufferObject::new(&device, &mode, format, modifier) {
					Ok(fbo) => {
						unsafe {
							(mem.as_mut_ptr() as *mut FrameBufferObject).add(i).write(fbo)
						}
					},
					Err(err) => {
						for i in 0 .. i {
							unsafe {
								std::mem::drop(
									(mem.as_mut_ptr() as *mut FrameBufferObject).add(i).read()
								);
							}
						}
						return Err(err)
					}
				}
			}

			unsafe { mem.assume_init() }
		};
		*/

		/*
		let framebuffers: [FrameBufferObject; FRAMEBUFFERS] = {
			use std::mem::MaybeUninit;

			let mut mem = unsafe { MaybeUninit::<[MaybeUninit<FrameBufferObject>; FRAMEBUFFERS]>::uninit().assume_init() };

			for i in 0 .. FRAMEBUFFERS {
				match FrameBufferObject::new(&device, &mode, format, modifier) {
					Ok(fbo) => { mem[i].write(fbo); },
					Err(err) => {
						for i in 0 .. i {
							unsafe { mem[i].assume_init_drop() }
						}
						return Err(err)
					}
				}
			}

			unsafe {
				(&mem as *const _ as *const [FrameBufferObject; FRAMEBUFFERS]).read()
			}
		};
		*/

		Ok(
			KmsContext {
				device,
				connector,
				mode,
				crtc,
				plane,
				property_cache
			}
		)
	}

	pub fn create_swapchain(
		&self,
		framebuffer_count: usize,
		format: DrmFourcc,
		modifier: DrmModifier,
		old_swapchain: Option<KmsSwapchain>
	) -> anyhow::Result<KmsSwapchain> {
		let is_first_frame = match old_swapchain {
			None => true,
			Some(ref old_swapchain) => old_swapchain.is_first_frame
		};
		std::mem::drop(old_swapchain);

		let mut framebuffers = Vec::with_capacity(framebuffer_count);
		for _ in 0 .. framebuffer_count {
			let fbo = FrameBufferObject::new(self.device.clone(), &self.mode, format, modifier)?;
			framebuffers.push(fbo);
		}

		Ok(
			KmsSwapchain {
				framebuffers,
				current_index: 0,
				is_first_frame
			}
		)
	}

	fn atomic_commit(
		&self,
		allow_modeset: bool,
		fbo: &FrameBufferObject
	) -> anyhow::Result<()> {
		use drm::control::atomic::{AtomicCommitFlags, AtomicModeReq};

		// let mut flags = AtomicCommitFlags::NONBLOCK;
		/*
		* DRM_MODE_PAGE_FLIP_EVENT signalizes that we want to receive a
		* page-flip event in the DRM-fd when the page-flip happens. This flag
		* is also used in the non-atomic examples, so you're probably familiar
		* with it.
		*
		* DRM_MODE_ATOMIC_NONBLOCK makes the page-flip non-blocking. We don't
		* want to be blocked waiting for the commit to happen, since we can use
		* this time to prepare a new framebuffer, for instance. We can only do
		* this because there are mechanisms to know when the commit is complete
		* (like page flip event, explained above).
		*/
		let mut flags = AtomicCommitFlags::empty();
		let mut request = AtomicModeReq::new();

		if allow_modeset {
			request.add_property(self.connector.handle(), self.property_cache.connector_crtc_id, self.crtc.info.handle().into());
			request.add_property(self.crtc.handle(), self.property_cache.crtc_mode_id, self.property_cache.blob_mode);
			request.add_property(self.crtc.handle(), self.property_cache.crtc_active, PropertyValue::Boolean(true));

			flags |= AtomicCommitFlags::ALLOW_MODESET;
		}

		request.add_property(self.plane.handle(), self.property_cache.plane_fb_id, fbo.framebuffer().into());
		request.add_property(self.plane.handle(), self.property_cache.plane_crtc_id, self.crtc.info.handle().into());
		request.add_property(self.plane.handle(), self.property_cache.plane_src_x, 0.into());
		request.add_property(self.plane.handle(), self.property_cache.plane_src_y, 0.into());
		request.add_property(self.plane.handle(), self.property_cache.plane_src_w, ((self.mode.size().0 as u64) << 16).into());
		request.add_property(self.plane.handle(), self.property_cache.plane_src_h, ((self.mode.size().1 as u64) << 16).into());
		request.add_property(self.plane.handle(), self.property_cache.plane_crtc_x, PropertyValue::SignedRange(0));
		request.add_property(self.plane.handle(), self.property_cache.plane_crtc_y, PropertyValue::SignedRange(0));
		request.add_property(self.plane.handle(), self.property_cache.plane_crtc_w, (self.mode.size().0 as u64).into());
		request.add_property(self.plane.handle(), self.property_cache.plane_crtc_h, (self.mode.size().1 as u64).into());

		self.device.atomic_commit(flags, request).context("Failed to perform atomic commit")?;

		Ok(())
	}

	pub fn device(&self) -> &KmsDevice {
		&self.device
	}

	pub fn resolution(&self) -> [usize; 2] {
		[self.mode.size().0 as usize, self.mode.size().1 as usize]
	}
}

pub struct KmsSwapchain {
	framebuffers: Vec<FrameBufferObject>,
	current_index: usize,
	is_first_frame: bool
}
impl KmsSwapchain {
	pub fn swap(&mut self) {
		self.current_index = (self.current_index + 1) % self.framebuffers.len();
	}

	pub fn current_framebuffer(&self) -> (usize, &FrameBufferObject) {
		(self.current_index, &self.framebuffers[self.current_index])
	}

	pub fn present(
		&mut self,
		context: &KmsContext
	) -> anyhow::Result<()> {
		context.atomic_commit(self.is_first_frame, self.current_framebuffer().1)?;
		self.is_first_frame = false;

		Ok(())
	}
}
