// Copyright (c) 2021 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use cgmath::Vector2;
use rand::Rng;
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        PrimaryCommandBufferAbstract,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::Queue,
    image::ImageAccess,
    memory::allocator::StandardMemoryAllocator,
    pipeline::{ComputePipeline, Pipeline, PipelineBindPoint},
    sync::GpuFuture,
};
use vulkano_util::renderer::DeviceImageView;

use arbitrary_fixed::ArbitraryFixed;

//use crate::arbitrary_fixed::ArbitraryFixed;

pub struct FractalComputePipeline {
    queue: Arc<Queue>,
    pipeline: Arc<ComputePipeline>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    palette: Arc<CpuAccessibleBuffer<[[f32; 4]]>>,
    palette_size: i32,
    end_color: [f32; 4],
}

impl FractalComputePipeline {
    pub fn new(
        queue: Arc<Queue>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    ) -> FractalComputePipeline {
        // Initial colors
        let colors = vec![
            [1.0, 0.0, 0.0, 1.0],
            [1.0, 1.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
            [0.0, 1.0, 1.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
            [1.0, 0.0, 1.0, 1.0],
        ];
        let palette_size = colors.len() as i32;
        let palette = CpuAccessibleBuffer::from_iter(
            &memory_allocator,
            BufferUsage {
                storage_buffer: true,
                ..BufferUsage::empty()
            },
            false,
            colors,
        )
        .unwrap();
        let end_color = [0.0, 0.0, 0.0, 1.0];

        let pipeline = {
            let shader = cs::load(queue.device().clone()).unwrap();
            ComputePipeline::new(
                queue.device().clone(),
                shader.entry_point("main").unwrap(),
                &(),
                None,
                |_| {},
            )
            .unwrap()
        };

        FractalComputePipeline {
            queue,
            pipeline,
            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
            palette,
            palette_size,
            end_color,
        }
    }

    /// Randomizes our color palette
    pub fn randomize_palette(&mut self) {
        let mut colors = vec![];
        for _ in 0..self.palette_size {
            let r = rand::thread_rng().gen::<f32>();
            let g = rand::thread_rng().gen::<f32>();
            let b = rand::thread_rng().gen::<f32>();
            //let a = rand::thread_rng().gen::<f32>();
            colors.push([r, g, b, 1.0]);
        }
        self.palette = CpuAccessibleBuffer::from_iter(
            &self.memory_allocator,
            BufferUsage {
                storage_buffer: true,
                ..BufferUsage::empty()
            },
            false,
            colors.into_iter(),
        )
        .unwrap();
    }

    pub fn compute(
        &self,
        image: DeviceImageView,
        c: Vector2<f32>,
        scale: ArbitraryFixed,
        translation: Vector2<ArbitraryFixed>,
        max_iters: u32,
        is_julia: bool,
    ) -> Box<dyn GpuFuture> {
        // Resize image if needed
        let img_dims = image.image().dimensions().width_height();
        let pipeline_layout = self.pipeline.layout();
        let desc_layout = pipeline_layout.set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            desc_layout.clone(),
            [
                WriteDescriptorSet::image_view(0, image),
                WriteDescriptorSet::buffer(1, self.palette.clone()),
            ],
        )
        .unwrap();
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let push_constants = cs::ty::PushConstants {
            c: c.into(),
            scale: scale.data,
            translation_x: translation.x.data,
            translation_y: translation.y.data,
            //translation: translation.into(),
            end_color: self.end_color,
            palette_size: self.palette_size,
            max_iters: max_iters as i32,
            is_julia: is_julia as u32,
            _dummy0: [0u8; 8], // Required for alignment
        };
        builder
            .bind_pipeline_compute(self.pipeline.clone())
            .bind_descriptor_sets(PipelineBindPoint::Compute, pipeline_layout.clone(), 0, set)
            .push_constants(pipeline_layout.clone(), 0, push_constants)
            .dispatch([img_dims[0] / 8, img_dims[1] / 8, 1])
            .unwrap();
        let command_buffer = builder.build().unwrap();
        let finished = command_buffer.execute(self.queue.clone()).unwrap();
        finished.then_signal_fence_and_flush().unwrap().boxed()
    }
}

include!(concat!(env!("OUT_DIR"), "/cs.rs"));
