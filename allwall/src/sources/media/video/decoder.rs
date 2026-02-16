use std::fmt;

use gstreamer::prelude::*;

use super::error::VideoError;
use crate::prelude::*;

pub struct Decoder {
    pipeline: gstreamer::Pipeline,
    appsink: gstreamer_app::AppSink,
}

impl fmt::Debug for Decoder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Decoder")
            .field("pipeline", &"gstreamer::Pipeline")
            .field("appsink", &"gstreamer_app::AppSink")
            .finish()
    }
}

impl Decoder {
    pub fn new(video_path: &std::path::Path) -> Result<Self> {
        let path_str = video_path
            .to_str()
            .ok_or_else(|| VideoError::FileNotFound(video_path.to_path_buf()))?;

        if !video_path.exists() {
            return Err(VideoError::FileNotFound(video_path.to_path_buf()).into());
        }

        let pipeline_str = format!(
            "filesrc location={} ! decodebin3 ! videoconvert ! video/x-raw,format=RGBA ! appsink name=sink emit-signals=true",
            path_str
        );

        let pipeline = gstreamer::parse::launch(&pipeline_str)
            .map_err(|e| VideoError::PipelineParse(e.to_string()))?
            .downcast::<gstreamer::Pipeline>()
            .map_err(|_| VideoError::PipelineDowncast)?;

        let appsink = pipeline
            .by_name("sink")
            .ok_or(VideoError::SinkNotFound("sink"))?
            .dynamic_cast::<gstreamer_app::AppSink>()
            .map_err(|_| VideoError::SinkCast)?;

        appsink.set_callbacks(
            gstreamer_app::AppSinkCallbacks::builder()
                .new_sample(|appsink| {
                    let _sample = appsink.pull_sample().map_err(|_| gstreamer::FlowError::Eos)?;
                    Ok(gstreamer::FlowSuccess::Ok)
                })
                .build(),
        );

        pipeline
            .set_state(gstreamer::State::Playing)
            .map_err(|e| VideoError::PipelineStart(e.to_string()))?;

        Ok(Self { pipeline, appsink })
    }

    pub fn appsink(&self) -> &gstreamer_app::AppSink {
        &self.appsink
    }

    pub fn pipeline(&self) -> &gstreamer::Pipeline {
        &self.pipeline
    }

    pub fn pull_sample(&self) -> Option<gstreamer::Sample> {
        let timeout = gstreamer::ClockTime::from_mseconds(16);
        self.appsink.try_pull_sample(timeout)
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gstreamer::State::Null);
    }
}
