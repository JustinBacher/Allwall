mod fps;
mod next;
mod prev;
pub mod protocol;

use clap::Subcommand;
pub use fps::Fps;
pub use next::Next;
pub use prev::Prev;

#[derive(Subcommand, Debug)]
pub enum IpcCommand {
    /// Skip to next image/video
    Next(Next),

    /// Go to previous image/video
    Prev(Prev),

    /// Set the target framerate
    Fps(Fps),
}
