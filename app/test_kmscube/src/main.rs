use drm::buffer::{DrmFourcc, DrmModifier};

mod kms;
mod egl;

fn main() {
	edwardium_logger::Logger::new(
		edwardium_logger::targets::stderr::StderrTarget::new(log::Level::Debug, Default::default()),
		std::time::Instant::now()
	).init_boxed().expect("Failed to initialize logger");

	let kms = kms::KmsContext::new(
		"/dev/dri/card0"
	).expect("Failed to initialize drm context");

	let format = DrmFourcc::Xrgb8888;

	let egl = egl::EglContext::new(&kms, format).expect("Failed to initialize egl");

	let mut swapchain = kms.create_swapchain(
		2,
		format,
		DrmModifier::Linear,
		None
	).expect("Failed to create kms swapchain");

	// let [width, height] = kms.resolution();
	// let max_radius = width.max(height) as f32;

	let mut current_frame: usize = 0;
	let mut stats_start = (0, std::time::Instant::now());
	// let mut frame_image = vec![0u8; width * height * 4];

	loop {
		// let frame_radius = (current_frame as f32) / 600.0 * max_radius;

		// for (i, pixel) in frame_image.as_mut_slice().chunks_mut(4).enumerate() {
		// 	let x = i % width;
		// 	let y = i / width;
		// 	let radius = ((x as f32).powi(2) + (y as f32).powi(2)).sqrt();

		// 	pixel[0] = if radius <= frame_radius { 0 } else { 255 };
		// 	pixel[1] = (y & 0xFF) as u8;
		// 	pixel[2] = (x & 0xFF) as u8;
		// }

		{

		}

		// swapchain.current_framebuffer().buffer_mut().write(&frame_image).unwrap().expect("Failed to map buffer object");
		// TODO: render

		swapchain.present(&kms).expect("Failed to present");
		swapchain.swap();

		current_frame += 1;
		if stats_start.1.elapsed() >= std::time::Duration::from_secs(1) || current_frame >= 600 {
			let elapsed_time = stats_start.1.elapsed();
			let elapsed_frames = current_frame - stats_start.0;
			log::debug!("Average fps: {}", elapsed_frames as f32 / elapsed_time.as_secs_f32());

			if current_frame >= 600 {
				break;
			}

			stats_start = (current_frame, std::time::Instant::now());
		}
	}
}
