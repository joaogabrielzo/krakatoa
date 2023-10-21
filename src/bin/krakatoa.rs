use anyhow::Result;
use ash::vk;
use krakatoa::camera::Camera;
use krakatoa::krakatoa::Krakatoa;
use krakatoa::model::{InstanceData, Model};
use nalgebra::{Matrix4, Vector3};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

fn main() -> Result<()> {
    /* Window */
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Krakatoa")
        .build(&event_loop)?;
    let mut krakatoa = Krakatoa::init(window)?;
    let mut cube = Model::cube();
    cube.insert_visibly(InstanceData {
        model_matrix: (Matrix4::new_translation(&Vector3::new(0.0, 0.0, 0.1))
            * Matrix4::new_scaling(0.1))
        .into(),
        colour: [0.2, 0.4, 1.0],
    });
    cube.insert_visibly(InstanceData {
        model_matrix: (Matrix4::new_translation(&Vector3::new(0.05, 0.05, 0.0))
            * Matrix4::new_scaling(0.1))
        .into(),
        colour: [1.0, 1.0, 0.2],
    });
    for i in 0..10 {
        for j in 0..10 {
            cube.insert_visibly(InstanceData {
                model_matrix: (Matrix4::new_translation(&Vector3::new(
                    i as f32 * 0.2 - 1.0,
                    j as f32 * 0.2 - 1.0,
                    0.5,
                )) * Matrix4::new_scaling(0.03))
                .into(),
                colour: [1.0, i as f32 * 0.07, j as f32 * 0.07],
            });
            cube.insert_visibly(InstanceData {
                model_matrix: (Matrix4::new_translation(&Vector3::new(
                    i as f32 * 0.2 - 1.0,
                    0.0,
                    j as f32 * 0.2 - 1.0,
                )) * Matrix4::new_scaling(0.02))
                .into(),
                colour: [i as f32 * 0.07, j as f32 * 0.07, 1.0],
            });
        }
    }
    cube.insert_visibly(InstanceData {
        model_matrix: (Matrix4::from_scaled_axis(Vector3::new(0.0, 0.0, 1.4))
            * Matrix4::new_translation(&Vector3::new(0.0, 0.5, 0.0))
            * Matrix4::new_scaling(0.1))
        .into(),
        colour: [0.0, 0.5, 0.0],
    });
    cube.insert_visibly(InstanceData {
        model_matrix: (Matrix4::new_translation(&Vector3::new(0.5, 0.0, 0.0))
            * Matrix4::new_nonuniform_scaling(&Vector3::new(0.5, 0.01, 0.01)))
        .into(),
        colour: [1.0, 0.5, 0.5],
    });
    cube.insert_visibly(InstanceData {
        model_matrix: (Matrix4::new_translation(&Vector3::new(0.0, 0.5, 0.0))
            * Matrix4::new_nonuniform_scaling(&Vector3::new(0.01, 0.5, 0.01)))
        .into(),
        colour: [0.5, 1.0, 0.5],
    });

    cube.insert_visibly(InstanceData {
        model_matrix: (Matrix4::new_translation(&Vector3::new(0.0, 0.0, 0.0))
            * Matrix4::new_nonuniform_scaling(&Vector3::new(0.01, 0.01, 0.5)))
        .into(),
        colour: [0.5, 0.5, 1.0],
    });

    cube.update_vertex_buffer(
        &krakatoa.logical_device,
        krakatoa.physical_device_memory_properties,
    )?;
    cube.update_instance_buffer(
        &krakatoa.logical_device,
        krakatoa.physical_device_memory_properties,
    )?;

    krakatoa.models = vec![cube];

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
                winit::event::VirtualKeyCode::Right => {
                    camera.turn_right(0.1);
                }
                winit::event::VirtualKeyCode::Left => {
                    camera.turn_left(0.1);
                }
                winit::event::VirtualKeyCode::Up => {
                    camera.move_forward(0.05);
                }
                winit::event::VirtualKeyCode::Down => {
                    camera.move_backward(0.05);
                }
                winit::event::VirtualKeyCode::PageUp => {
                    camera.turn_up(0.02);
                }
                winit::event::VirtualKeyCode::PageDown => {
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
