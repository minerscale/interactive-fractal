#version 450

#include "arbitraryfixed.glsl"

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

// Image to which we'll write our fractal
layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

// Our palette as a dynamic buffer
layout(set = 0, binding = 1) buffer Palette {
    vec4 data[];
} palette;

// Our variable inputs as push constants
layout(push_constant) uniform PushConstants {
    vec2 c;
    vec2 scale;
    uint translation_x[SIZE];
    uint translation_y[SIZE];
    vec4 end_color;
    int palette_size;
    int max_iters;
    bool is_julia;
} push_constants;

// Gets smooth color between current color (determined by iterations) and the next color in the palette
// by linearly interpolating the colors based on: https://linas.org/art-gallery/escape/smooth.html
vec4 get_color(
    int palette_size,
    vec4 end_color,
    int i,
    int max_iters,
    float len_z
) {
    if (i < max_iters) {
        float iters_float = float(i) + 1.0 - log(log(len_z)) / log(2.0f);
        float iters_floor = floor(iters_float);
        float remainder = iters_float - iters_floor;
        vec4 color_start = palette.data[int(iters_floor) % push_constants.palette_size];
        vec4 color_end = palette.data[(int(iters_floor) + 1) % push_constants.palette_size];
        return mix(color_start, color_end, remainder);
    }
    return end_color;
}

void main() {
    // Scale image pixels to range
    ivec2 dims = imageSize(img);
    uint ar[SIZE];
    fix_from_float(ar, float(dims.x) / float(dims.y));

    //float x_over_width = (gl_GlobalInvocationID.x / dims.x);
    //float y_over_height = (gl_GlobalInvocationID.y / dims.y);
    uint tmp_0[SIZE];
    fix_from_float(tmp_0, float(gl_GlobalInvocationID.x));
    fix_divide_by_u32(tmp_0, tmp_0, dims.x);
    uint tmp_1[SIZE];
    fix_from_float(tmp_1, float(gl_GlobalInvocationID.y));
    fix_divide_by_u32(tmp_1, tmp_1, dims.y);

    //float x0 = ar * (push_constants.translation.x + (x_over_width - 0.5) * push_constants.scale.x);
    //float y0 = push_constants.translation.y + (y_over_height - 0.5) * push_constants.scale.y;

    uint tmp_3[SIZE];
    fix_from_float(tmp_3, -0.5);
    uint tmp_4[SIZE];
    fix_add(tmp_4, tmp_0, tmp_3);
    fix_from_float(tmp_0, push_constants.scale.x);
    fix_mul(tmp_0, tmp_4, tmp_0);
    fix_add(tmp_0, tmp_0, push_constants.translation_x);
    fix_mul(tmp_0, tmp_0, ar);

    fix_add(tmp_4, tmp_1, tmp_3);
    fix_from_float(tmp_1, push_constants.scale.y);
    fix_mul(tmp_1, tmp_4, tmp_1);
    fix_add(tmp_1, tmp_1, push_constants.translation_y);

    // Julia is like mandelbrot, but instead changing the constant `c` will change the shape
    // you'll see. Thus we want to bind the c to mouse position.
    // With mandelbrot, c = scaled xy position of the image. Z starts from zero.
    // With julia, c = any value between the interesting range (-2.0 - 2.0), Z = scaled xy position of the image.

    uint cx[SIZE];
    uint cy[SIZE];
    uint zx[SIZE];
    uint zy[SIZE];

    if (push_constants.is_julia) {
        //c = push_constants.c;
        fix_from_float(cx, push_constants.c.x);
        fix_from_float(cy, push_constants.c.y);
        //z = vec2(x0, y0);
        fix_copy(zx, tmp_0);
        fix_copy(zy, tmp_1);
    } else {
        //c = vec2(x0, y0);
        fix_copy(cx, tmp_0);
        fix_copy(cy, tmp_1);
        //z = vec2(0.0, 0.0);
        fix_from_float(zx, 0.0);
        fix_from_float(zy, 0.0);
    }

    // Escape time algorithm:
    // https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set
    // It's an iterative algorithm where the bailout point (number of iterations) will determine
    // the color we choose from the palette
    int i;
    //float len_z;
    uint len_z[SIZE];

    uint sixty_four[SIZE];
    fix_from_float(sixty_four, 64.0);
    for (i = 0; i < push_constants.max_iters; i += 1) {
        //z = vec2(
        //    z.x * z.x - z.y * z.y + c.x,
        //    z.y * z.x + z.x * z.y + c.y
        //);

        fix_mul(tmp_0, zx, zx);
        fix_mul(tmp_1, zy, zy);
        fix_sub(tmp_0, tmp_0, tmp_1);
        fix_add(tmp_3, tmp_0, cx);

        fix_mul(tmp_1, zx, zy);
        fix_add(tmp_0, tmp_1, tmp_1);
        fix_add(zy, tmp_0, cy);

        fix_copy(zx, tmp_3);

        // len_z = length(z);
        fix_mul(tmp_0, zx, zx);
        fix_mul(tmp_1, zy, zy);
        fix_add(len_z, tmp_0, tmp_1);
        
        // Using 8.0 for bailout limit give a little nicer colors with smooth colors
        // 2.0 is enough to 'determine' an escape will happen
        if (fix_compare(len_z, sixty_four) > 0) {
            break;
        }
    }

    float len_z_f32 = fix_to_float(len_z);

    vec4 write_color = get_color(
        push_constants.palette_size,
        push_constants.end_color,
        i,
        push_constants.max_iters,
        len_z_f32
    );
    imageStore(img, ivec2(gl_GlobalInvocationID.xy), write_color);
}
