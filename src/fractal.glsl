#version 450

#include "arbitraryfixed.glsl"

#define SSAA_SAMPLES 1

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
    uint scale[SIZE];
    uint translation_x[SIZE];
    uint translation_y[SIZE];
    vec4 end_color;
    int palette_size;
    int max_iters;
    bool is_julia;
} push_constants;

void main() {
    uint tmp_0[SIZE];
    uint tmp_1[SIZE];
    uint tmp_2[SIZE];

    uint cx[SIZE];
    uint cy[SIZE];
    uint zx[SIZE];
    uint zy[SIZE];

    uint px[SIZE];
    uint py[SIZE];

    // Scale image pixels to range
    ivec2 dims = imageSize(img);
    uint ar[SIZE];
    fix_from_float(ar, float(dims.x) / float(dims.y));

    //float x_over_width = (gl_GlobalInvocationID.x / dims.x);
    fix_from_float(tmp_0, float(gl_GlobalInvocationID.x) / float(dims.x));

    
    //float y_over_height = (gl_GlobalInvocationID.y / dims.y);
    fix_from_float(tmp_1, float(gl_GlobalInvocationID.y) / float(dims.y));

    //float x0 = ar * (push_constants.translation.x + (x_over_width - 0.5) * push_constants.scale.x);
    fix_from_float(tmp_2, -0.5);
    fix_add(tmp_0, tmp_0, tmp_2);
    fix_mul(tmp_0, tmp_0, push_constants.scale);
    fix_add(tmp_0, tmp_0, push_constants.translation_x);
    fix_mul(px, tmp_0, ar);

    //float y0 = push_constants.translation.y + (y_over_height - 0.5) * push_constants.scale.y;
    fix_add(tmp_1, tmp_1, tmp_2);
    fix_mul(tmp_1, tmp_1, push_constants.scale);
    fix_add(py, tmp_1, push_constants.translation_y);

    // Julia is like mandelbrot, but instead changing the constant `c` will change the shape
    // you'll see. Thus we want to bind the c to mouse position.
    // With mandelbrot, c = scaled xy position of the image. Z starts from zero.
    // With julia, c = any value between the interesting range (-2.0 - 2.0), Z = scaled xy position of the image.

    uint escape_squared[SIZE];
    fix_from_float(escape_squared, 64.0);

    uint px_mul[SIZE];
    fix_from_float(px_mul, 1.0/float(dims.y));
    fix_mul(px_mul, px_mul, push_constants.scale);

    vec4 write_color = vec4(0.0, 0.0, 0.0, 0.0);
    for (int k = 0; k < SSAA_SAMPLES * SSAA_SAMPLES; ++k) {
        fix_from_float(tmp_0, float(k % SSAA_SAMPLES)/float(SSAA_SAMPLES));
        fix_mul(tmp_0, tmp_0, px_mul);

        fix_from_float(tmp_1, float(k / SSAA_SAMPLES)/float(SSAA_SAMPLES));
        fix_mul(tmp_1, tmp_1, px_mul);

        if (push_constants.is_julia) {
            //c = push_constants.c;
            fix_from_float(cx, push_constants.c.x);
            fix_from_float(cy, push_constants.c.y);
            //z = vec2(x0, y0);
            fix_add(zx, px, tmp_0);
            fix_add(zy, py, tmp_1);
            //fix_copy(zx, px);
            //fix_copy(zy, py);
        } else {
            //c = vec2(x0, y0);
            fix_add(cx, px, tmp_0);
            fix_add(cy, py, tmp_1);
            //fix_copy(cx, px);
            //fix_copy(cy, py);
            //z = vec2(0.0, 0.0);
            fix_from_float(zx, 0.0);
            fix_from_float(zy, 0.0);
        }
    
        // Escape time algorithm:
        // https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set
        // It's an iterative algorithm where the bailout point (number of iterations) will determine
        // the color we choose from the palette
        int i;
    
        for (i = 0; i < push_constants.max_iters; i += 1) {
            //z = vec2(
            //    z.x * z.x - z.y * z.y + c.x,
            //    z.y * z.x + z.x * z.y + c.y
            //);
    
            fix_mul(tmp_0, zx, zx);
            fix_mul(tmp_1, zy, zy);
            fix_sub(tmp_0, tmp_0, tmp_1);
            fix_add(tmp_2, tmp_0, cx);
    
            fix_mul(tmp_1, zx, zy);
            fix_add(tmp_0, tmp_1, tmp_1);
            fix_add(zy, tmp_0, cy);
    
            fix_copy(zx, tmp_2);
    
            // len_z = length(z);
            fix_mul(tmp_0, zx, zx);
            fix_mul(tmp_1, zy, zy);
            fix_add(tmp_0, tmp_0, tmp_1);
            
            // Using 8.0 for bailout limit give a little nicer colors with smooth colors
            // 2.0 is enough to 'determine' an escape will happen
            fix_sub(tmp_1, tmp_0, escape_squared);
            
            if (!is_negative(tmp_1)) {
                break;
            }
        }

        if (i < push_constants.max_iters) {
            float iters_float = float(i) + 1.0 - log(log(sqrt(fix_to_float(tmp_0)))) / log(2.0f);
            float iters_floor = floor(iters_float);
            float remainder = iters_float - iters_floor;
            vec4 color_start = palette.data[int(iters_floor) % push_constants.palette_size];
            vec4 color_end = palette.data[(int(iters_floor) + 1) % push_constants.palette_size];
            vec4 color_mixed = mix(color_start, color_end, remainder);
            // Gamma corrected!!
            write_color += color_mixed * color_mixed;
        }
    }

    write_color.w = float(SSAA_SAMPLES * SSAA_SAMPLES);
    imageStore(img, ivec2(gl_GlobalInvocationID.xy), sqrt(write_color/(float(SSAA_SAMPLES * SSAA_SAMPLES))));
}
