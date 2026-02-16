pub mod still;
pub mod video;

use std::{path::PathBuf, time::Duration};

use rand::seq::SliceRandom;

use crate::{
    engine::{Context, Texture},
    prelude::*,
    sources::{BasicSource, RenderState, Source, SourceType, error::SourceError},
    transitions::TransitionType,
};

use self::still::Still;
use self::video::Video;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MediaKind {
    Image,
    Video,
}

fn detect_media_kind(path: &PathBuf) -> Option<MediaKind> {
    let mime = mime_guess::from_path(path).first()?;
    let type_ = mime.type_().as_str();
    match type_ {
        "image" => Some(MediaKind::Image),
        "video" => Some(MediaKind::Video),
        _ => None,
    }
}

pub enum MediaSource {
    Still(Still),
    Video(Video),
}

impl MediaSource {
    pub fn from_directory(dir: &PathBuf, ctx: &Context) -> Result<Self> {
        let mut rng = rand::rng();
        let mut files: Vec<_> = dir
            .read_dir()
            .map_err(|_| SourceError::NoImageDirectory)?
            .filter_map(std::result::Result::ok)
            .map(|d| d.path())
            .filter(|p| p.is_file())
            .collect();

        if files.is_empty() {
            return Err(SourceError::NoImagesAvailable.into());
        }

        files.shuffle(&mut rng);

        for path in files {
            match detect_media_kind(&path) {
                Some(MediaKind::Image) => {
                    if let Ok(img) = image::open(&path) {
                        let still = Still::new(&img, dir.clone(), ctx).with_current_path(path);
                        return Ok(Self::Still(still));
                    }
                },
                Some(MediaKind::Video) => {
                    if let Ok(video) = Video::new(path.clone(), dir.clone(), ctx) {
                        return Ok(Self::Video(video));
                    }
                },
                None => continue,
            }
        }

        Err(SourceError::NoImagesAvailable.into())
    }

    pub fn directory(&self) -> &PathBuf {
        match self {
            MediaSource::Still(s) => s.directory(),
            MediaSource::Video(v) => v.directory(),
        }
    }
}

impl Source for MediaSource {
    fn texture(&self) -> &Texture {
        match self {
            MediaSource::Still(s) => s.texture(),
            MediaSource::Video(v) => v.texture(),
        }
    }

    fn state(&self) -> &RenderState {
        match self {
            MediaSource::Still(s) => s.state(),
            MediaSource::Video(v) => v.state(),
        }
    }

    fn load(&mut self, ctx: &Context) -> Result<()> {
        match self {
            MediaSource::Still(s) => s.load(ctx),
            MediaSource::Video(v) => v.load(ctx),
        }
    }

    fn start_transition(
        &mut self,
        previous: Option<SourceType>,
        duration: Duration,
        ctx: &Context,
        transition_type: TransitionType,
    ) {
        match self {
            MediaSource::Still(s) => s.start_transition(previous, duration, ctx, transition_type),
            MediaSource::Video(v) => v.start_transition(previous, duration, ctx, transition_type),
        }
    }

    fn update(&mut self, dt: Duration) {
        match self {
            MediaSource::Still(s) => s.update(dt),
            MediaSource::Video(v) => v.update(dt),
        }
    }

    fn next(&self, ctx: &Context) -> Result<Self> {
        match self {
            MediaSource::Still(s) => s.next(ctx).map(MediaSource::Still),
            MediaSource::Video(_) => Err(SourceError::UnsupportedOperation("next for video".to_string()).into()),
        }
    }

    fn prev(&self, ctx: &Context) -> Result<Self> {
        match self {
            MediaSource::Still(s) => s.prev(ctx).map(MediaSource::Still),
            MediaSource::Video(_) => Err(SourceError::UnsupportedOperation("prev for video".to_string()).into()),
        }
    }
}

impl BasicSource for MediaSource {
    fn render(&mut self, ctx: &Context) {
        match self {
            MediaSource::Still(s) => s.render(ctx),
            MediaSource::Video(v) => v.render(ctx),
        }
    }
}

impl std::fmt::Debug for MediaSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaSource::Still(s) => s.fmt(f),
            MediaSource::Video(v) => v.fmt(f),
        }
    }
}
