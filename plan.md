This is a complex systems programming project. To ensure success, we will break this down into a **MVP (Minimum Viable Product)** approach.

We will start by building a standard windowed application to prove the data flow works. Once we confirm GStreamer is decoding and wgpu is rendering, we will swap the window handle for a Wayland Layer Shell surface.

**Prerequisite Hardware/Driver Setup**
Before writing code, ensure your system is ready for hardware decoding.

1. **Intel/AMD:** Ensure `libva` and `mesa` drivers are installed. Install `gstreamer1.0-vaapi`.
2. **NVIDIA:** Ensure proprietary drivers are installed. Install `gstreamer1.0-nvdec` (or similar depending on distro).
3. **GStreamer Plugins:** You need `base`, `good`, `bad`, and `ugly` plugins.

---

### Phase 1: Project Scaffold

Create a new Rust project and set up the dependencies.

1. **Initialize Project:**

   ```bash
   cargo init allwall --bin
   ```

2. **Update `Cargo.toml`:**
   We need async (Tokio) for the Wayland event loop, GStreamer for video, and Winit/Wgpu for the initial window.

   ```toml
   [dependencies]
   gstreamer = "0.22"
   gstreamer-app = "0.22"
   gstreamer-video = "0.22"
   wgpu = "0.20"
   winit = "0.29" # Used only in Phase 1-4
   bytemuck = "1.15" # Helper for casting bytes
   thiserror = "2.0.18"
   env_logger = "0.11.8"
   clap = "4.5.57"
   ```

3. **Basic `main.rs` Skeleton:**
   Create the file structure. We will use a struct to hold the application state.

   ```rust
   use gstreamer::prelude::*;
   use std::sync::Arc;

   struct VideoApp {
       pipeline: gstreamer::Pipeline,
       appsink: gstreamer_app::AppSink,
       // WGPU fields will be added in Phase 3
   }

   fn main() {
       // Init GStreamer
       gstreamer::init().expect("Failed to init GStreamer");
       // App logic will go here
   }
   ```

---

### Phase 2: GStreamer Hardware Decoding

We need a pipeline that decodes to a raw format. Ideally, we want DMABUF, but for the **initial MVP window**, we will ask for RGBA. GStreamer's `videoconvert` will handle the hardware-to-software bridge if necessary, ensuring the video _plays_.

_Note: To strictly enforce hardware only, we would use `video/x-raw(memory:DMABUF)`, but standard WGPU cannot import DMABUFs easily yet. We will implement the "Copy" method here to ensure you see a video first._

1. **Implement the Pipeline Builder:**

   ```rust
   impl VideoApp {
       fn new(video_path: &str) -> Self {
           // Construct pipeline.
           // We use decodebin3 for auto-plugging (including hardware decoders).
           // We force format=RGBA to make wgpu texture uploading easy for the MVP.
           let pipeline_str = format!(
               "filesrc location={} ! decodebin3 ! videoconvert ! video/x-raw,format=RGBA ! appsink name=sink emit-signals=true",
               video_path
           );

           let pipeline = gstreamer::Pipeline::parse(&pipeline_str)
               .expect("Failed to parse pipeline");

           let appsink = pipeline
               .by_name("sink")
               .expect("Sink not found")
               .dynamic_cast::<gstreamer_app::AppSink>()
               .expect("Sink is not AppSink");

           // Configure Appsink to pull buffers manually
           appsink.set_callbacks(
               gstreamer_app::AppSinkCallbacks::builder()
                   .build()
           );

           pipeline
               .set_state(gstreamer::State::Playing)
               .expect("Failed to start pipeline");

           Self { pipeline, appsink }
       }
   }
   ```

---

### Phase 3: Winit Window & WGPU Setup

Now we create a visible window and the graphics context.

1. **Update `main` to create the window and device:**

   ```rust
   fn main() {
       gstreamer::init().unwrap();

       let event_loop = winit::event_loop::EventLoop::new().unwrap();
       let window = winit::window::WindowBuilder::new()
           .with_title("Video Wallpaper MVP")
           .build(&event_loop)
           .unwrap();

       // WGPU Setup
       let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
           backends: wgpu::Backends::all(),
           ..Default::default()
       });
       let surface = instance.create_surface(window.clone()).unwrap();

       let adapter = instance
           .request_adapter(&wgpu::RequestAdapterOptions {
               power_preference: wgpu::PowerPreference::HighPerformance,
               compatible_surface: Some(&surface),
               ..Default::default()
           })
           .await
           .expect("No adapter found");

       let (device, queue) = adapter
           .request_device(
               &wgpu::DeviceDescriptor {
                   label: Some("Device"),
                   required_features: wgpu::Features::empty(),
                   required_limits: wgpu::Limits::default(),
               },
               None,
           )
           .await
           .unwrap();

       // ... we will plug this into the App struct in Phase 4 ...
   }
   ```

---

### Phase 4: Bridging & Rendering (The "See It" Phase)

This is where we connect the GStreamer buffer to the WGPU texture. We will perform a "Staging Copy". The data is read from GStreamer (CPU) and uploaded to the GPU. _This satisfies "get that to just a window"._

1. **Update `VideoApp` to handle WGPU state:**
   Add fields: `device: wgpu::Device`, `queue: wgpu::Queue`, `texture: Option<wgpu::Texture>`.

2. **The Frame Update Logic:**
   We need to poll GStreamer for a new sample. If it exists, we upload it to a wgpu texture.

   ```rust
   impl VideoApp {
       // Inside the App struct implementation
       fn update_frame(&mut self) -> Result<(), Box<dyn std::error::Error>> {
           // Try to get a sample (non-blocking, duration 0)
           if let Some(sample) = self.appsink.try_pull_sample(gstreamer::ClockTime::from_mseconds(0)) {
               let buffer = sample.buffer().ok_or("No buffer?")?;

               // Map the buffer readable (CPU side)
               let map = buffer.map_readable().ok_or("Buffer not readable")?;
               let data = map.as_slice();

               let info = buffer
                   .caps()
                   .and_then(|c| c.structure(0))
                   .ok_or("No caps")?;

               let width = info.get::<i32>("width")? as u32;
               let height = info.get::<i32>("height")? as u32;

               // Create or Resize Texture
               let texture_desc = wgpu::TextureDescriptor {
                   label: Some("Video Texture"),
                   size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                   mip_level_count: 1,
                   sample_count: 1,
                   dimension: wgpu::TextureDimension::D2,
                   format: wgpu::TextureFormat::Rgba8UnormSrgb, // Matches GStreamer RGBA
                   usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                   view_formats: &[],
               };

               let texture = self.device.create_texture(&texture_desc);

               // Upload Data to GPU
               self.queue.write_texture(
                   wgpu::ImageCopyTexture {
                       texture: &texture,
                       mip_level: 0,
                       origin: wgpu::Origin3d::ZERO,
                       aspect: wgpu::TextureAspect::All,
                   },
                   data,
                   wgpu::ImageDataLayout {
                       offset: 0,
                       bytes_per_row: Some(4 * width),
                       rows_per_image: Some(height),
                   },
                   texture_desc.size,
               );

               self.texture = Some(texture);
               self.size = Some((width, height));
           }
           Ok(())
       }
   }
   ```

3. **The Render Loop:**
   Standard WGPU render pass using a simple shader. (I will assume you know how to create a basic Vertex buffer for a fullscreen quad).

   ```rust
   // In the event loop run
   event_loop.run(move |event, _| {
       match event {
           winit::event::Event::WindowEvent { event: winit::event::WindowEvent::CloseRequested, .. } => {
               // Exit
           }
           winit::event::Event::AboutToWait => {
               // 1. Update texture from GStreamer
               let _ = app.update_frame();

               // 2. Render
               if let (Some(surface), Some(texture)) = (&surface, &app.texture) {
                    // ... Wgpu Render Pass setup ...
                    // set_pipeline, set_bind_group(0, &texture_bind_group), draw...
               }
               window.request_redraw();
           }
           _ => {}
       }
   }).unwrap();
   ```

**Checkpoint:** Run this. You should see your video playing in a floating window. The zero-copy constraint is relaxed here (we are doing CPU-GPU copy) to prove the pipeline works.

---

### Phase 5: Moving to Wayland Wallpaper (Background Surface)

Once the video plays in the window, we replace the Window with a Wayland Layer Shell.

1. **Update Dependencies:**
   Remove `winit`. Add `smithay-client-toolkit`.

   ```toml
   [dependencies]
   smithay-client-toolkit = "0.18"
   wayland-client = "0.31"
   wayland-backend = "0.3"
   # Remove winit
   ```

2. **Setup Wayland Display:**
   Instead of `winit::EventLoop`, we connect to the Wayland compositor.

   ```rust
   use smithay_client_toolkit::reexports::protocols::wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer;
   use smithay_client_toolkit::shell::wlr_layer::{LayerShell, LayerSurface};

   fn main() {
       // ... GStreamer init ...

       // Connect to Wayland
       let display = wayland_client::Display::connect_to_env().unwrap();

       // Setup Event Queue (We will run this manually instead of winit's)
       let mut event_queue = display.create_event_queue();
       let qh = event_queue.handle();

       // Get Globals
       let registry = display.get_registry(&qh, &qh);

       // ... We need to wait for the registry to announce the Layer Shell ...
       // Usually you'd run a roundtrip here.
   }
   ```

3. **Initialize Layer Shell:**
   We need a surface that sits "behind" all windows.

   ```rust
   // Inside the main setup, after registry announces compositor & layer_shell
   let layer_shell = LayerShell::bind(&display, &qh).unwrap();

   let surface = display.create_surface(&qh, &qh);

   let layer_surface = layer_shell.create_layer_surface(
       &surface,
       Layer::Background, // This puts it behind windows
       "video-wallpaper",
       None,
       &qh,
   );

   // Ensure it covers the screen
   // You'll likely need to listen for Output events to get screen resolution
   layer_surface.set_size(1920, 1080);
   layer_surface.set_exclusive_zone(-1); // Full coverage
   ```

4. **Create WGPU Surface from Wayland:**
   This is the crucial switch. `wgpu` has a generic `SurfaceTarget`.

   ```rust
   // Use the raw Wayland display and surface to create WGPU surface
   let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
   let surface = unsafe {
       instance.create_surface::<wgpu::SurfaceTargetUnsafe>(
           wgpu::SurfaceTargetUnsafe::Wayland(
               display.get_display_ptr() as *mut _, // Raw pointer
               surface.id().as_ptr() as *mut _,
           )
       )
   }.expect("Failed to create surface from Wayland");
   ```

5. **The Loop:**
   Wayland doesn't have a blocking "event loop" like Winit. You have a file descriptor to listen on. You usually pair this with a timer for your rendering framerate.

   ```rust
   // Pseudo-code for the loop
   loop {
       // 1. Dispatch pending Wayland events
       event_queue.dispatch_pending(&mut (), &mut ()).unwrap();

       // 2. Update GStreamer Frame
       app.update_frame();

       // 3. Render to the WGPU Surface (Same as Phase 4)
       render_frame(&app, &surface);

       // 4. Wait a bit (FPS cap) or poll for new events
       // In a real app you might use calloop or similar
       std::thread::sleep(std::time::Duration::from_millis(16));
   }
   ```

### Summary of Deliverables

1. **MVP (Phase 4):** A binary that opens a window and plays a video using GStreamer -> Wgpu. This confirms the data extraction and rendering logic works.
2. **Final (Phase 5):** A binary that runs on Wayland, opens no window, but renders the video to the background via `wlr-layer-shell`.

**Note on "True" Zero-Copy:**
To move from the MVP (Phase 4) to the Zero-Copy goal, you must replace the `queue.write_texture` call in Phase 4 with a `wgpu-hal` texture import. This requires using `ash` (Vulkan) to import the DMABUF file descriptor from GStreamer directly. This is a risky refactor if the basic window doesn't work first, which is why we validate the pipeline with a copy first.
