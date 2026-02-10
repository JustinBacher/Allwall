use crate::prelude::*;

use gstreamer::prelude::*;

pub struct Decoder {
	pipeline: gstreamer::Pipeline,
	appsink: gstreamer_app::AppSink,
}

impl Decoder {
	pub fn new(video_path: &str) -> Result<Self> {
		let pipeline_str = format!(
			"filesrc location={} ! decodebin3 ! videoconvert ! video/x-raw,format=RGBA ! appsink name=sink emit-signals=true",
			video_path
		);

		let pipeline = gstreamer::parse::launch(&pipeline_str)
			.map_err(|e| Error::Generic(f!("Failed to parse pipeline: {:?}", e)))?
			.downcast::<gstreamer::Pipeline>()
			.map_err(|_| Error::Static("Pipeline is not a Pipeline object"))?;

		let appsink = pipeline
			.by_name("sink")
			.ok_or(Error::Static("Sink not found"))?
			.dynamic_cast::<gstreamer_app::AppSink>()
			.map_err(|_| Error::Static("Sink is not AppSink"))?;

		appsink.set_callbacks(gstreamer_app::AppSinkCallbacks::builder().build());

		pipeline
			.set_state(gstreamer::State::Playing)
			.map_err(|e| Error::Generic(f!("Failed to start pipeline: {:?}", e)))?;

		Ok(Self { pipeline, appsink })
	}

	pub fn appsink(&self) -> &gstreamer_app::AppSink {
		&self.appsink
	}
}

impl Drop for Decoder {
	fn drop(&mut self) {
		let _ = self.pipeline.set_state(gstreamer::State::Null);
	}
}
