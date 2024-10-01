// Camera information holding the camera's position and it's (inverse) view-projection matrix.
struct CameraUniform {
    position: vec4<f32>,
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>
}

// Obsolete
struct Voxel {
    color: u32
}

// VoxelGrid storing how large the Voxel Grid is.
struct VoxelGrid {
    dimensions: vec4<u32>,
    box_min: vec4<f32>,
    box_size: vec4<f32>,
    // Buffer is only needed for WGSL byte alignment and is not used further,
    buffer: vec4<f32>
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
}

// Hit information for a single point in space where a position was sampled along a ray.
struct HitInfo {
    hit: bool,
    color: vec3<f32>,
    alpha: f32,
}

// Raymarching Output information holding the resulting data for a single marched ray.
// Basically: Multiple HitInfos + Alpha Blending will result in this struct.
struct RayMarchOutput {
    hit: bool,
    color: vec4<f32>,
    distance: f32,
    steps: u32,
    min_distance_to_scene: f32,
}

// Holds information about a intersection with a Axis-Aligned-Bounding-Box
// Only useful in context with the ray that was used to perform the test
struct AABBIntersection {
    intersects: bool,
    t_min: f32,
    t_max: f32,
}


// Vertex function, as we use a single clip-stace triangle stretched across the entire window we don't do anything here
@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Camera information
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// Obsolete
// TODO: Remove this as it wastes a lot of VRAM and is already stored in the voxel texture
@group(1) @binding(1)
var<storage, read> voxels: array<Voxel>;

// VoxelGrid information that tells us how large the volume is
@group(1) @binding(0)
var<uniform> voxel_grid: VoxelGrid;

// Volume texture and it's sampler that contains the volume information
@group(2) @binding(0)
var voxel_texture: texture_3d<f32>;

@group(2) @binding(1)
var voxel_texture_sampler: sampler;

const MAX_STEP_AMOUNT: i32 = 5000;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Center NDC coordinates to the center of the screen
    var screen_position = vec4<f32>(in.tex_coords.x, in.tex_coords.y, 1.0, 1.0);
    screen_position -= vec4<f32>(0.5, 0.5, 0.0, 0.0);
    screen_position *= vec4<f32>(2.0, 2.0, 1.0, 1.0);

    // Using the screenposition and the inverse view-projection-matrix, calculate the direction of that particular pixel
    let world_position = camera.inverse_view_proj * screen_position;
    let dir: vec3<f32> = normalize(world_position.xyz);
    var ro: vec3<f32> = camera.position.xyz;

    // Raymarch into the scene
    let raymarch_result = raymarch(ro, dir);
    // let rel_red = vec4<f32>(f32(raymarch_result.steps) / f32(MAX_STEP_AMOUNT), 0.0, 0.0, 1.0);
    return vec4<f32>(raymarch_result.color);
    // return rel_red;
}

// Raymarch function that takes a ray's origin and its direction and samples the scene at specific points along the ray's direction
fn raymarch(ro: vec3<f32>, rd: vec3<f32>) -> RayMarchOutput {
    var output: RayMarchOutput = RayMarchOutput();

    // Set initial colors and alpha for alpha blending
    var res = vec4<f32>(0.0);
    var color = vec3<f32>(0.0);
    var alpha = 0.0;
    let step_size = 0.001;

    output.min_distance_to_scene = 10000.0;

    // Check if the ray ever intersects the volume texture and exit out early if it doesn't
    let aabb_intersection = aabb_intersect(ro, rd, voxel_grid.box_min.xyz, voxel_grid.box_size.xyz);
    if !aabb_intersection.intersects {
        output.color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        return output;
    }

    // Set initial ray distance to the first point where the ray intersects with the volume
    var dt = aabb_intersection.t_min;
    // var dt = 0.0;
    for(var i = 0; i < MAX_STEP_AMOUNT; i += 1) {
        // Calculate next position & then sample the scene at that point
        let p: vec3<f32> = ro + rd * dt;
        let hitInfo = scene(p);

        // Use front-to-back alpha blending
        var alpha_src = 1.0 - exp(-hitInfo.alpha * step_size * voxel_grid.buffer[0]);
        // var alpha_src = hitInfo.alpha / 2000.0;
        var color_src = hitInfo.color;
        color = color + (1.0 - alpha) * alpha_src * color_src;
        alpha = alpha + (1.0 - alpha) * alpha_src;

        // var color: vec4<f32> = vec4<f32>(hitInfo.color, hitInfo.alpha);
        // color = alpha * (hitInfo.alpha * hitInfo.color) + color;
        // alpha = (1 - hitInfo.alpha) * alpha;

        // var color: vec4<f32> = vec4<f32>(hitInfo.color, hitInfo.alpha * 1000.0);
        // color *= vec4<f32>(vec3<f32>(color.a), 1.0);
        // res += color * (1.0 - res.a);
        // Debugging: Uncomment to highlight center as a red sphere
        // if length(vec3<f32>(16.0) - p) < 0.5 {
        //     output.color = vec4<f32>(1.0,0.0, 0.0, 1.0);
        //     alpha = 1.0;
        //     return output;
        // }

        // When the alpha reaches 1.0, no more color from behind has an influence on the output image so we stop raymarching
        if(alpha >= 1.0) {
            break;
        }

        // Increase distance for the next sampling step
        dt += step_size;
        output.steps = output.steps + 1;

        if dt >= aabb_intersection.t_max {
            break;
        }
    }

    // Background color
    // color = alpha * color + (1.0 - alpha) * vec3<f32>(0.0);
    // alpha = alpha + (1.0 - alpha);

    output.color = vec4<f32>(color, alpha);
    output.distance = dt;
    return output;
}

// Samples the scene at a specific point in space
fn scene(p: vec3<f32>) -> HitInfo {
    var output: HitInfo = HitInfo();
    let box_min = voxel_grid.box_min.xyz;
    let box_max = voxel_grid.box_min.xyz + voxel_grid.box_size.xyz;
    let dimensions = voxel_grid.dimensions.xyz;
    var p_r = p + vec3<f32>(voxel_grid.box_size.xyz);
    let p_trunc: vec3<i32> = vec3<i32>(trunc(p_r));
    let fract: vec3<f32> = fract(p_r);
    let pos: vec3<i32> = p_trunc;

    let rel_p = (p - voxel_grid.box_min.xyz) / voxel_grid.box_size.xyz;


    output.alpha = 0.0;
    output.color = vec3<f32>(0.0);


    // Check if the point is inside or outside the dimensions of the box
    // NOTE: This should be obsolete with the AABB Intersection Test
    if p.x < box_min.x || p.y < box_min.y || p.z < box_min.z {
        return output;
    }
    if p.x >= box_max.x || p.y >= box_max.y || p.z >= box_max.z {
        return output;
    }

    // Get relative coordinates inside the box and sample the volume texture
    // let texture_coords = p_r / vec3<f32>(dimensions);
    let sample_result = textureSample(voxel_texture, voxel_texture_sampler, rel_p);

    // Get relative color relative to a 1x1x1 grid
    // var sample_result = vec3<f32>(rel_p);
    output.alpha = sample_result.a;
    // Adjust alpha for a more interesting appearance
    // output.alpha = sample_result.a;

    output.hit = true;
    output.color = sample_result.rgb;
    return output;
}

// Adapted from https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-box-intersection.html
// Checks for a intersection with a AABB. The origin is always at (0,0,0) and the furthest corner is at (box_size.x, box_size.y, box_size.z).
// Returns if the intersection hit, the minimum and maximum distance the ray has to travel for the intersections with the box's boundaries.
fn aabb_intersect(ro: vec3<f32>, rd: vec3<f32>, box_min: vec3<f32>, box_size: vec3<f32>) -> AABBIntersection {
    var intersection = AABBIntersection();
    let min = box_min;
    let max = box_min + box_size;

    var t_min = (min.x - ro.x) / rd.x;
    var t_max = (max.x - ro.x) / rd.x;

    if t_min > t_max {
        let temp = t_min;
        t_min = t_max;
        t_max = temp;
    }

    var t_y_min = (min.y - ro.y) / rd.y;
    var t_y_max = (max.y - ro.y) / rd.y;

    if t_y_min > t_y_max {
        let temp = t_y_min;
        t_y_min = t_y_max;
        t_y_max = temp;
    }

    if (t_min > t_y_max) || (t_y_min > t_max) { 
        intersection.intersects = false;
        return intersection;
    }

    if t_y_min > t_min { t_min = t_y_min; }
    if t_y_max < t_max { t_max = t_y_max; }

    var t_z_min = (min.z - ro.z) / rd.z;
    var t_z_max = (max.z - ro.z) / rd.z;

    if t_z_min > t_z_max {
        let temp = t_z_min;
        t_z_min= t_z_max;
        t_z_max = temp;
    }

    if (t_min > t_z_max) || (t_z_min > t_max) { 
        intersection.intersects = false;
        return intersection;
    }

    if t_z_min > t_min { t_min = t_z_min; }
    if t_z_max < t_max { t_max = t_z_max; }

    intersection.intersects = true;
    intersection.t_min = t_min;
    intersection.t_max = t_max;
    return intersection;
}

// Obsolete
fn get_1d_index(p: vec3<i32>) -> i32{
    return p.x + i32(voxel_grid.dimensions.x) * (p.y + i32(voxel_grid.dimensions.y) * p.z);
}

// Obsolete
fn get_voxel_alpha(voxel: Voxel) -> u32 {
    return voxel.color & 0xFF;
}