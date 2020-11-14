extern crate futures;
extern crate imgui;
extern crate imgui_winit_support;
extern crate wgpu;
extern crate winit;

mod simple_error;

use imgui::*;
use imgui_winit_support::*;
use std::error::Error;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use simple_error::*;

//glslangValidator -V shader.vert -o shader.vert.spv

fn render(context: &mut Context) -> Result<(), Box<dyn Error>> {
    let frame = context.swap_chain.get_current_frame()?.output;

    let imgui = &mut context.imgui;
    context
        .imgui_platform
        .prepare_frame(imgui.io_mut(), &context.window)?;

    let ui = imgui.frame();

    {
        let window = imgui::Window::new(im_str!("Hello world"));
        window
            .size([300.0, 100.0], Condition::FirstUseEver)
            .build(&ui, || {
                ui.text(im_str!("Hello world!"));
                ui.text(im_str!("This...is...imgui-rs on WGPU!"));
                ui.separator();
                let mouse_pos = ui.io().mouse_pos;
                ui.text(im_str!(
                    "Mouse Position: ({:.1},{:.1})",
                    mouse_pos[0],
                    mouse_pos[1]
                ));
            });

        let window = imgui::Window::new(im_str!("Hello too"));
        window
            .size([400.0, 200.0], Condition::FirstUseEver)
            .position([400.0, 200.0], Condition::FirstUseEver)
            .build(&ui, || {
                ui.text(im_str!("Frametime: {:?}", 100));
            });

        ui.show_demo_window(&mut context.demo_open);
    }

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

        let renderer = &mut context.imgui_renderer;
        renderer.render(ui.render(), &context.queue, &context.device, &mut render_pass)
        .map_err(|_| SimpleError::new("Error rendering imgui"))?;
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
    imgui: imgui::Context,
    imgui_platform: imgui_winit_support::WinitPlatform,
    imgui_renderer: imgui_wgpu::Renderer,
    demo_open: bool,
}

async fn setup(window: Window) -> Result<Context, Box<dyn Error>> {
    //set up wgpu
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

    //set up imgui
    let hidpi_factor = window.scale_factor();
    let mut imgui = imgui::Context::create();
    let mut platform = WinitPlatform::init(&mut imgui);
    platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Default);
    imgui.set_ini_filename(None);

    let font_size = (13.0 * hidpi_factor) as f32;
    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    imgui
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

    //set up imgui_wgpu
    let renderer_config = imgui_wgpu::RendererConfig {
        texture_format: swap_chain_descriptor.format,
        ..Default::default()
    };

    let renderer = imgui_wgpu::Renderer::new(&mut imgui, &device, &queue, renderer_config);

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
        imgui,
        imgui_renderer: renderer,
        imgui_platform: platform,
        demo_open: true
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

        &context.imgui_platform.handle_event(context.imgui.io_mut(), &context.window, &event);
    });
}

fn main() -> Result<(), Box<dyn Error>> {
    futures::executor::block_on(run())
}
