mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/fractal.glsl",
        include: [INCLUDE_PATH],
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}
