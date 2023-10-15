use anyhow::Result;
use ash::vk;
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
    let mut angle = 0.2;
    let my_special_cube = cube.insert_visibly(InstanceData {
        model_matrix: (Matrix4::from_scaled_axis(Vector3::new(0.0, 0.0, angle))
            * Matrix4::new_translation(&Vector3::new(0.0, 0.5, 0.0))
            * Matrix4::new_scaling(0.1))
        .into(),
        colour: [0.0, 0.5, 0.0],
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

    use winit::event::{Event, WindowEvent};
    event_loop.run(move |event, _, controlflow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *controlflow = winit::event_loop::ControlFlow::Exit;
        }
        Event::MainEventsCleared => {
            angle += 0.01;
            krakatoa.models[0]
                .get_mut(my_special_cube)
                .unwrap()
                .model_matrix = (Matrix4::new_translation(&Vector3::new(0.0, 0.5, 0.0))
                * Matrix4::from_scaled_axis(Vector3::new(0.0, 0.0, angle))
                * Matrix4::new_scaling(0.1))
            .into();
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

                krakatoa
                    .logical_device
                    .reset_fences(&[
                        krakatoa.swapchain.may_begin_drawing[krakatoa.swapchain.current_image]
                    ])
                    .expect("Resetting fences.")
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
