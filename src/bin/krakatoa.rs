use anyhow::Result;
use ash::vk;
use krakatoa::camera::Camera;
use krakatoa::krakatoa::Krakatoa;
use krakatoa::model::{InstanceData, Model};
use nalgebra::Matrix4;
use winit::event::VirtualKeyCode;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

fn main() -> Result<()> {
    /* Window */
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Krakatoa")
        .build(&event_loop)?;
    let mut krakatoa = Krakatoa::init(window)?;
    let mut sphere = Model::sphere(3);
    sphere.insert_visibly(InstanceData {
        model_matrix: Matrix4::new_scaling(0.5).into(),
        colour: [0.5, 0.0, 0.0],
    });

    sphere.update_vertex_buffer(
        &krakatoa.logical_device,
        krakatoa.physical_device_memory_properties,
    )?;
    sphere.update_index_buffer(
        &krakatoa.logical_device,
        krakatoa.physical_device_memory_properties,
    )?;
    sphere.update_instance_buffer(
        &krakatoa.logical_device,
        krakatoa.physical_device_memory_properties,
    )?;

    krakatoa.models = vec![sphere];

    let mut camera = Camera::builder().build();

    use winit::event::{Event, WindowEvent};
    event_loop.run(move |event, _, controlflow| match event {
        Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } => match input {
            winit::event::KeyboardInput {
                state: winit::event::ElementState::Pressed,
                virtual_keycode: Some(keycode),
                ..
            } => match keycode {
                VirtualKeyCode::Right | VirtualKeyCode::D => {
                    camera.turn_right(0.1);
                }
                VirtualKeyCode::Left | VirtualKeyCode::A => {
                    camera.turn_left(0.1);
                }
                VirtualKeyCode::Up | VirtualKeyCode::W => {
                    camera.move_forward(0.05);
                }
                VirtualKeyCode::Down | VirtualKeyCode::S => {
                    camera.move_backward(0.05);
                }
                VirtualKeyCode::PageUp | VirtualKeyCode::Q => {
                    camera.turn_up(0.02);
                }
                VirtualKeyCode::PageDown | VirtualKeyCode::E => {
                    camera.turn_down(0.02);
                }
                _ => {}
            },
            _ => {}
        },
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *controlflow = winit::event_loop::ControlFlow::Exit;
        }
        Event::MainEventsCleared => {
            krakatoa.window.request_redraw();
        }
        Event::RedrawRequested(_) => {
            krakatoa.swapchain.current_image =
                (krakatoa.swapchain.current_image + 1) % krakatoa.swapchain.amount_of_images;

            let (image_index, _) = unsafe {
                krakatoa
                    .swapchain
                    .swapchain_loader
                    .acquire_next_image(
                        krakatoa.swapchain.swapchain,
                        std::u64::MAX,
                        krakatoa.swapchain.image_available[krakatoa.swapchain.current_image],
                        vk::Fence::null(),
                    )
                    .expect("Image acquisition failed.")
            };

            unsafe {
                krakatoa
                    .logical_device
                    .wait_for_fences(
                        &[krakatoa.swapchain.may_begin_drawing[krakatoa.swapchain.current_image]],
                        true,
                        std::u64::MAX,
                    )
                    .expect("Waiting fences.");

                krakatoa
                    .logical_device
                    .reset_fences(&[
                        krakatoa.swapchain.may_begin_drawing[krakatoa.swapchain.current_image]
                    ])
                    .expect("Resetting fences.");

                camera.update_buffer(
                    &krakatoa.logical_device,
                    krakatoa.physical_device_memory_properties,
                    &mut krakatoa.uniform_buffer,
                );

                krakatoa.models.iter_mut().for_each(|m| {
                    m.update_instance_buffer(
                        &krakatoa.logical_device,
                        krakatoa.physical_device_memory_properties,
                    )
                    .expect("Updating instance buffer.")
                });

                krakatoa
                    .update(image_index as usize)
                    .expect("Updating the command buffer.");
            }

            let semaphores_available =
                [krakatoa.swapchain.image_available[krakatoa.swapchain.current_image]];
            let waiting_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let semaphores_finished =
                [krakatoa.swapchain.rendering_finished[krakatoa.swapchain.current_image]];
            let command_buffers = [krakatoa.command_buffers[image_index as usize]];
            let submit_info = [vk::SubmitInfo::builder()
                .wait_semaphores(&semaphores_available)
                .wait_dst_stage_mask(&waiting_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&semaphores_finished)
                .build()];
            unsafe {
                krakatoa
                    .logical_device
                    .queue_submit(
                        krakatoa.queues.graphics_queue,
                        &submit_info,
                        krakatoa.swapchain.may_begin_drawing[krakatoa.swapchain.current_image],
                    )
                    .expect("Queue submission.");
            };

            let swapchains = [krakatoa.swapchain.swapchain];
            let indices = [image_index];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&semaphores_finished)
                .swapchains(&swapchains)
                .image_indices(&indices);
            unsafe {
                krakatoa
                    .swapchain
                    .swapchain_loader
                    .queue_present(krakatoa.queues.graphics_queue, &present_info)
                    .expect("Queue presentation.");
            }
        }
        _ => {}
    });
}
