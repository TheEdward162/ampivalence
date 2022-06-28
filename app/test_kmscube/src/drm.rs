use std::{fs, path::Path, iter, os::unix::prelude::{AsRawFd, RawFd}};

use anyhow::Context;
use drm::{
	Device,
	ClientCapability,
	control::{
		Device as ControlDevice,
		Mode, ModeTypeFlags,
		connector::{Handle as ConnectorHandle, State as ConnectorState, Info as ConnectorInfo},
		encoder::{Handle as EncoderHandle, Info as EncoderInfo},
		crtc::{Handle as CrtcHandle, Info as CrtcInfo},
		plane::{Info as PlaneInfo},
		property::{Value as PropertyValue}
	}
};
use gbm::Device as GbmDevice;

struct IndexedCrtc {
	pub info: CrtcInfo,
	pub index: usize
}
impl std::fmt::Debug for IndexedCrtc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let raw_handle: u32 = self.info.handle().into();
		write!(f, "Crtc#{}({})", self.index, raw_handle)
    }
}

struct DrmDevice(fs::File);
impl DrmDevice {
	fn choose_connector(&self, connectors: &[ConnectorHandle]) -> anyhow::Result<ConnectorInfo> {
		let mut chosen = None;
		let mut found_multiple = false;

		for &handle in connectors {
			let connector = self.get_connector(handle).context("Failed to query connector")?;
			log::trace!("Connector: {:?}#{} [{:?}]", connector.interface(), connector.interface_id(), connector.state());

			if connector.state() == ConnectorState::Connected {
				if chosen.is_none() {
					chosen = Some(connector);
				} else {
					found_multiple = true;
				}
			}
		}

		if found_multiple {
			log::warn!("Found multiple connected connectors. Choosing first one.")
		}

		match chosen {
			None => Err(anyhow::anyhow!("Did not find any connected connectors")),
			Some(connector) => {
				log::info!("Choosing connector: {:?}#{}", connector.interface(), connector.interface_id());
				Ok(connector)
			}
		}
	}

	fn choose_mode(&self, modes: &[Mode]) -> anyhow::Result<Mode> {
		let mut chosen = None;
		
		for &mode in modes {
			log::trace!(
				"Mode: \"{}\" {}x{}@{} [{:?}]",
				mode.name().to_str().unwrap_or("<Unknown>"),
				mode.size().0, mode.size().1,
				mode.vrefresh(),
				mode.mode_type()
			);

			chosen = match chosen.take() {
				None => Some(mode),
				Some(current) => {
					if mode.mode_type().contains(ModeTypeFlags::PREFERRED) {
						Some(mode)
					} else if current.mode_type().contains(ModeTypeFlags::PREFERRED) {
						Some(current)
					} else {
						let mode_area = mode.size().0 * mode.size().1;
						let current_area = current.size().0 * current.size().1;

						if mode_area > current_area {
							Some(mode)
						} else if mode_area < current_area {
							Some(current)
						} else if mode.vrefresh() > current.vrefresh() {
							Some(mode)
						} else {
							Some(current)
						}
					}
				}
			};
		}

		match chosen {
			None => Err(anyhow::anyhow!("Did not find any modes for chosen connector")),
			Some(mode) => {
				log::info!(
					"Choosing mode: \"{}\" {}x{}@{} [{:?}]",
					mode.name().to_str().unwrap_or("<Unknown>"),
					mode.size().0, mode.size().1,
					mode.vrefresh(),
					mode.mode_type()
				);
				Ok(mode)
			}
		}
	}

	fn choose_crtc(
		&self,
		current_encoder: Option<EncoderHandle>,
		encoders: &[Option<EncoderHandle>],
		crtcs: &[CrtcHandle]
	) -> anyhow::Result<IndexedCrtc> {
		let mut chosen = None;

		for handle in iter::once(current_encoder).chain(encoders.into_iter().copied()) {
			let handle = match handle {
				None => continue,
				Some(handle) => handle
			};
			
			let encoder = self.get_encoder(handle).context("Failed to query encoder")?;
			log::trace!("Encoder: {:?} ({:?})", encoder.kind(), encoder.crtc());

			if chosen.is_none() {
				chosen = self.choose_crts_for_encoder(encoder, crtcs)?;
			}
		}

		match chosen {
			None => Err(anyhow::anyhow!("Did not find any crtcs for chosen mode and available encoders")),
			Some(crtc) => {
				log::info!("Choosing crtc: {:?}", crtc);

				Ok(crtc)
			}
		}
	}

	fn choose_crts_for_encoder(&self, encoder: EncoderInfo, crtcs: &[CrtcHandle]) -> anyhow::Result<Option<IndexedCrtc>> {
		let mut chosen = None;

		match encoder.crtc() {
			None => (),
			Some(handle) => {
				let crtc = self.get_crtc(handle).context("Failed to query crtc")?;
				log::trace!("Crtc: {:?}", crtc);
				chosen = Some(crtc);
			}
		}

		for handle in encoder.possible_crtcs().filter_iter(crtcs.iter().copied()) {
			let crtc = self.get_crtc(handle).context("Failed to query crtc")?;
			log::trace!("Crtc: {:?}", crtc);

			if chosen.is_none() {
				chosen = Some(crtc);
			}
		}

		match chosen {
			None => Ok(None),
			Some(crtc) => {
				// we need to find the index of this crtc
				let index = crtcs.iter().enumerate().find_map(
					|(index, handle)| if *handle == crtc.handle() {
						Some(index)
					} else {
						None
					}
				).unwrap();

				let result = IndexedCrtc {
					info: crtc,
					index
				};

				Ok(Some(result))
			}
		}
	}

	fn choose_plane(&self, crtc: &IndexedCrtc) -> anyhow::Result<PlaneInfo> {
		let mut chosen = None;
		
		let planes = self.plane_handles().context("Failed to query plane resources")?;
		for &handle in planes.planes() {
			let plane = self.get_plane(handle).context("Failed to query plane")?;
			log::trace!("Plane: {:?}", plane);

			let mut is_primary = false;

			let properties = self.get_properties(handle).context("Failed to query plane properties")?;
			let (prop_handles, prop_values) = properties.as_props_and_values();
			for (handle, value) in prop_handles.into_iter().copied().zip(prop_values.into_iter().copied()) {
				let property = self.get_property(handle).context("Failed to query property")?;
				
				log::trace!(
					"Property: {}{}{:?}: {:?} = {}",
					if property.atomic() { "atomic " } else { "" },
					if property.mutable() { "mut " } else { "" },
					property.name(),
					property.value_type(),
					value
				);

				match property.name().to_str() {
					Ok("type") => match property.value_type().convert_value(value) {
						PropertyValue::Enum(Some(enum_value)) => match enum_value.name().to_str() {
							Ok("Primary") => { is_primary = true; }
							_ => ()
						},
						_ => { log::warn!("Unpexpected value type for property \"type\""); }
					},
					_ => ()
				}
			}

			// check that plane is compatible with the crtc
			if plane.possible_crtcs().check(crtc.index) {
				chosen = match chosen {
					None => Some((plane, is_primary)),
					Some((current, current_is_primary)) => {
						if is_primary {
							Some((plane, is_primary))
						} else {
							Some((current, current_is_primary))
						} 
					}
				}
			}
		}

		match chosen {
			None => Err(anyhow::anyhow!("Did not find any planes for chosen crtc")),
			Some((plane, is_primary)) => {

				log::info!(
					"Choosing plane: {:?}{}",
					plane.handle(),
					if is_primary { " [Primary]" } else { "" }
				);
				Ok(plane)
			}
		}
	}
}
impl std::os::unix::io::AsRawFd for DrmDevice {
	fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
		self.0.as_raw_fd()
	}
}
impl Device for DrmDevice {}
impl ControlDevice for DrmDevice {}

pub struct DrmContext {
	device: GbmDevice<DrmDevice>,
	connector: ConnectorInfo,
	mode: Mode,
	crtc: IndexedCrtc,
	plane: PlaneInfo
}
impl DrmContext {
	pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
		let file = fs::OpenOptions::new().read(true).write(true).open(path).context("Failed to open drm device")?;

		let device = GbmDevice::new(DrmDevice(file)).context("Failed to create gbm device")?;

		let resource_handles = device.resource_handles().context("Failed to query control device resources")?;
		let connector = device.choose_connector(resource_handles.connectors())?;
		let mode = device.choose_mode(connector.modes())?;
		let crtc = device.choose_crtc(
			connector.current_encoder(),
			connector.encoders(),
			resource_handles.crtcs()
		)?;

		device.set_client_capability(ClientCapability::Atomic, true).context("Failed to set Atomic client capability")?;
		let plane = device.choose_plane(&crtc)?;
		
		Ok(DrmContext {
			device,
			connector,
			mode,
			crtc,
			plane
		})
	}
}
