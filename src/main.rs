use std::{sync::{mpsc::{self, Receiver, Sender}, Arc, Mutex}};
use wgpu::core::device;
use winit::{
    event::{self, *},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    keyboard::{self, Key, KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

#[derive(PartialEq)]
enum KeyFlag {
    Pressed,
    Released,
    Exit
}

struct KeyState {
    key_flag: KeyFlag,
    keycode: KeyCode
}

fn process_window_event(event: &WindowEvent, control_flow: &EventLoopWindowTarget<()>, sender: Sender<KeyState>) {
    if let WindowEvent::CloseRequested{..} = event {
        sender.send(KeyState { key_flag: KeyFlag::Exit, keycode: KeyCode::Escape });
        control_flow.exit();
    }

    if let WindowEvent::KeyboardInput{ event: KeyEvent { state: ElementState::Pressed, physical_key, ..}, .. } = event {
        if let PhysicalKey::Code(keycode) = physical_key {
            sender.send(KeyState {key_flag: KeyFlag::Pressed, keycode: keycode.clone()});
        }
    }

    if  let WindowEvent::KeyboardInput{ event: KeyEvent { state: ElementState::Released, physical_key, ..}, .. } = event {
        if let PhysicalKey::Code(keycode) = physical_key {
            sender.send(KeyState {key_flag: KeyFlag::Released, keycode: keycode.clone()});
        }
    } 
}    

enum Entities {
    Player1,
    Player2
}

struct GameState {
    keyboard: [bool; 193],
    entities: Vec<Entities>,
    position: Vec<i32>,
    health: Vec<i32>,
}

impl GameState {
    fn new() -> Self {
        return Self {
            keyboard: [false; 193],
            entities: vec![],
            position: vec![],
            health: vec![]
        };
    }

    async fn run(&mut self, command_receiver: &mut Receiver<KeyState>) {
        loop {
            // Update keyboard state
            if let Ok(v) = command_receiver.try_recv() {
                if v.key_flag == KeyFlag::Exit {
                    break;
                }

                if v.key_flag == KeyFlag::Pressed {
                    self.keyboard[v.keycode as usize] = true;
                }

                if v.key_flag == KeyFlag::Released {
                    self.keyboard[v.keycode as usize] = false;
                }
            }

            if self.keyboard[KeyCode::KeyW as usize] == true {
                println!("forward!!");
            }

            if self.keyboard[KeyCode::KeyS as usize] == true {
                println!("backward");
            }

            if self.keyboard[KeyCode::KeyA as usize] == true {
                println!("left");
            }

            if self.keyboard[KeyCode::KeyD as usize] == true {
                println!("right");
            }
        }
    }
}

struct GraphicEngine<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: &'a Window,
    pipeline: wgpu::RenderPipeline
}

impl<'a> GraphicEngine<'a> {
    async fn new(window: &'a Window) -> GraphicEngine<'a> {
        let size = window.inner_size();
        // Instance corresponds to WebGPU's GPU object.
        // We specify the backend options we want in the instance descriptor, 
        // where primary can be anything (Metal, DX12, ) etc.
        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor{
                backends: wgpu::Backends::PRIMARY,
                ..Default::default()
            }
        );
        // This is our canvas
        // Why does borrowing the window trigger E0515??
        let surface = instance.create_surface(window).unwrap();

        // Corresponds to WebGPU's GPUAdapter
        // Gives us information about the system's implementation
        // of WebGPU?
        // Anyway, it's used to create the device object which we use to interface with the physical GPU
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptionsBase { 
                power_preference: wgpu::PowerPreference::default(), 
                force_fallback_adapter: false, 
                compatible_surface: Some(&surface)
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None
            },
            None,
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor { 
                label: Some("Render Pipeline Layout"), 
                bind_group_layouts: &[], 
                push_constant_ranges: &[]
            }
        );
        
        let pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[]
                },
                fragment: Some(wgpu::FragmentState { 
                    module: &shader, 
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL
                    })],
                }),
            }
        )


            
        return GraphicEngine {
            surface,
            device,
            queue,
            config,
            size,
            window
        };
    }
    
    // This function is called whenever a change in size is detected from
    // the window events.
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        return false;
    }

    fn window(&self) -> &Window {
        return &self.window;
    }

    fn update(&mut self) {

    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // I wait for surface to give me a new texture.
        // With this new texture, I can render stuff to it.
        // This texture will be stored in ououytput
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("Render Encder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    // Networking section begin

    // Networking section end

    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let (tx, mut rx) = mpsc::channel();

    tokio::spawn(async move {
        let mut game_state = GameState::new();
        game_state.run(&mut rx)
        .await
    });

    // Graphics section starts here

    let mut surface_configured = false;

    let mut state = GraphicEngine::new(&window).await;
    event_loop.run(move |event, control_flow| match event {
        Event::WindowEvent {ref event, window_id} if window_id == state.window.id() => {
            match event {
                WindowEvent::Resized(physical_size) => {
                    surface_configured = true;
                    state.resize(*physical_size);
                },

                WindowEvent::RedrawRequested => {
                    // Tells winit we want another frame
                    state.window().request_redraw();

                    if !surface_configured {
                        return;
                    }

                    state.update();
                    match state.render() {
                        Ok(_) => {},
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                        Err(e) => eprintln!("{:?}", e)
                    }
                },
                _ => { 
                    process_window_event(event, control_flow, tx.clone());
                }
            }
        },
        _ => ()
    }).expect("hi");
}
