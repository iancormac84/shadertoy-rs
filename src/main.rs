extern crate futures;
extern crate wgpu;
extern crate winit;

mod simple_error;

use std::error::Error;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use simple_error::*;

//glslangValidator -V shader.vert -o shader.vert.spv

fn render(context: &mut Context) -> Result<(), Box<dyn Error>> {
    let frame = context.swap_chain.get_current_frame()?.output;
    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
        render_pass.set_pipeline(&context.render_pipeline);
        render_pass.draw(0..3, 0..1);
    }

    context.queue.submit(Some(encoder.finish()));

    Ok(())
}

#[allow(dead_code)]
struct Context {
    window: Window,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    vertex_shader: wgpu::ShaderModule,
    fragment_shader: wgpu::ShaderModule,
    pipeline_layout: wgpu::PipelineLayout,
    render_pipeline: wgpu::RenderPipeline,
    swap_chain_descriptor: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    queue: wgpu::Queue,
}

async fn setup(window: Window) -> Result<Context, Box<dyn Error>> {
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
        .ok_or_else(|| SimpleError::new("Could not find appropriate adapater"))?;

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

    let swap_chain_descriptor = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: swap_chain_format,
        width: window_size.width,
        height: window_size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    let swap_chain = device.create_swap_chain(&surface, &swap_chain_descriptor);

    Ok(Context {
        window,
        surface,
        adapter,
        device,
        vertex_shader,
        fragment_shader,
        pipeline_layout,
        render_pipeline,
        swap_chain_descriptor,
        swap_chain,
        queue,
    })
}
async fn run() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("WGPU Experiments")
        .build(&event_loop)?;

    let mut context = setup(window).await?;

    event_loop.run(move |event, _target, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                context.swap_chain_descriptor.width = size.width;
                context.swap_chain_descriptor.height = size.height;
                context.swap_chain = context
                    .device
                    .create_swap_chain(&context.surface, &context.swap_chain_descriptor)
            }
            Event::MainEventsCleared => {
                match render(&mut context) {
                    Ok(()) => {} //render successful
                    Err(error) => {
                        println!("Encountered error: {}", error);
                        *control_flow = ControlFlow::Exit;
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
