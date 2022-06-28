mod drm;

fn main() {
	edwardium_logger::Logger::new(
		edwardium_logger::targets::stderr::StderrTarget::new(log::Level::Trace, Default::default()),
		std::time::Instant::now()
	).init_boxed().expect("Could not initialize logger");

	let drm_device = drm::DrmContext::new("/dev/dri/card0").expect("Could not initialize drm context");
}
