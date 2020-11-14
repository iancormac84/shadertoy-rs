extern crate futures;
extern crate wgpu;
extern crate winit;

use std::error::Error;
use std::fmt::{Display, Formatter};
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

//glslangValidator -V shader.vert -o shader.vert.spv

#[derive(Debug)]
struct StringError {
    message: &'static str,
}

impl StringError {
    fn new(message: &'static str) -> StringError {
        StringError { message }
    }
}

impl Display for StringError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for StringError {}

fn render(
    device: &wgpu::Device,
    swap_chain: &mut wgpu::SwapChain,
    queue: &wgpu::Queue,
    render_pipeline: &wgpu::RenderPipeline,
) -> Result<(), Box<dyn Error>> {
    let frame = swap_chain.get_current_frame()?.output;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("WGPU Command Encoder Descriptor"),
    });
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(render_pipeline);
        render_pass.draw(0..3, 0..1);
    }

    queue.submit(Some(encoder.finish()));

    Ok(())
}

async fn run() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("WGPU Experiments")
        .build(&event_loop)?;

    let window_size = window.inner_size();

    let swap_chain_format = wgpu::TextureFormat::Bgra8UnormSrgb;

    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };

    println!("Found these adapters:");
    for adapter in instance.enumerate_adapters(wgpu::BackendBit::PRIMARY) {
        println!("  {:?}", adapter.get_info());
    }

    println!();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
        })
        .await
        .ok_or_else(|| StringError::new("Could not find appropriate adapater"))?;

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                shader_validation: true,
            },
            None,
        )
        .await?;

    println!("Adapter: {:?}", adapter.get_info());
    println!("Device: {:?}", device);

    let vertex_shader = device.create_shader_module(wgpu::include_spirv!("shader.vert.spv"));
    let fragment_shader = device.create_shader_module(wgpu::include_spirv!("shader.frag.spv"));

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("WGPU Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("WGPU Pipeline"),
        layout: Some(&pipeline_layout),
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &vertex_shader,
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: &fragment_shader,
            entry_point: "main",
        }),
        rasterization_state: None,
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[swap_chain_format.into()],
        depth_stencil_state: None,
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });

    let mut swap_chain_descriptor = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: swap_chain_format,
        width: window_size.width,
        height: window_size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &swap_chain_descriptor);

    event_loop.run(move |event, _target, control_flow| {
        //really move all resources into this closure.
        let _ = (
            &adapter,
            &device,
            &vertex_shader,
            &fragment_shader,
            &pipeline_layout,
            &swap_chain_descriptor,
        );

        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                swap_chain_descriptor.width = size.width;
                swap_chain_descriptor.height = size.height;
                swap_chain = device.create_swap_chain(&surface, &swap_chain_descriptor)
            }
            Event::MainEventsCleared => {
                match render(&device, &mut swap_chain, &queue, &render_pipeline) {
                    Ok(()) => {} //render successful
                    Err(error) => {
                        println!("Encountered error: {}", error);
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("Exiting...");
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

fn main() -> Result<(), Box<dyn Error>> {
    futures::executor::block_on(run())
}
