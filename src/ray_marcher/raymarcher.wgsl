struct CameraUniform {
    position: vec4<f32>,
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>
}

struct Voxel {
    color: u32
}

struct VoxelGrid {
    dimensions: vec3<u32>,
    buffer: u32
}

struct HitInfo {
    hit: bool,
    color: vec3<f32>,
    alpha: f32,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
}

struct RayMarchOutput {
    hit: bool,
    color: vec3<f32>,
    distance: f32,
    steps: u32,
    min_distance_to_scene: f32,
}


@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 1.0);

    return out;
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(1)
var<storage, read> voxels: array<Voxel>;

@group(1) @binding(0)
var<uniform> voxel_grid: VoxelGrid;

@group(2) @binding(0)
var voxel_texture: texture_3d<f32>;

@group(2) @binding(1)
var voxel_texture_sampler: sampler;


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Center NDC coordinates to the center of the screen
    var screen_position = vec4<f32>(in.tex_coords.x, in.tex_coords.y, 1.0, 1.0);
    screen_position -= vec4<f32>(0.5, 0.5, 0.0, 0.0);

    // Using the screenposition and the inverse view-projection-matrix, calculate the direction of that particular pixel
    let world_position = camera.inverse_view_proj * screen_position;
    let dir: vec3<f32> = world_position.xyz;
    var ro: vec3<f32> = camera.position.xyz;

    // Raymarch into the scene
    let raymarch_result = raymarch(ro, dir);
    return vec4<f32>(raymarch_result.color, 1.0);
}

fn raymarch(ro: vec3<f32>, rd: vec3<f32>) -> RayMarchOutput {
    var output: RayMarchOutput = RayMarchOutput();
    var dt = 0.0;
    var color = vec3<f32>(1.0);
    var alpha = 0.0;

    output.min_distance_to_scene = 10000.0;
    for(var i = 0; i < 10000; i += 1) {
        // Sample scene at the current position
        let p: vec3<f32> = ro + rd * dt;
        let hitInfo = scene(p);

        // Use front-to-back alpha blending
        color = alpha * color + (1.0 - alpha) * hitInfo.alpha * hitInfo.color;
        alpha = alpha + (1.0 - alpha) * hitInfo.alpha;

        // When the alpha reaches 1.0, no more color from behind has an influence on the output image so we stop raymarching
        if(alpha >= 1.0) {
            break;
        }

        dt += 0.01;
        output.steps = output.steps + 1;
    }


    color = alpha * color + (1.0 - alpha) * vec3<f32>(1.0);
    alpha = alpha + (1.0 - alpha);

    output.color = color;
    output.distance = dt;
    return output;
}

fn scene(p: vec3<f32>) -> HitInfo {
    var output: HitInfo = HitInfo();
    let dimensions = voxel_grid.dimensions;
    let p_trunc: vec3<i32> = vec3<i32>(trunc(p));
    let fract: vec3<f32> = fract(p);
    let pos: vec3<i32> = p_trunc;
    // let index = get_1d_index(pos);
    // output.distance = 0.01;
    output.alpha = 0.0;
    output.color = vec3<f32>(1.0);


    if p.x < 0.0 || p.y < 0.0 || p.z < 0.0 {
        return output;
    }
    if pos.x >= i32(dimensions.x) || pos.y >= i32(dimensions.y) || pos.z >= i32(dimensions.z) {
        return output;
    }

    let texture_coords = p / vec3<f32>(dimensions);
    let sample_result = textureSample(voxel_texture, voxel_texture_sampler, texture_coords);

    // let voxel = voxels[index];
    var relative_color = vec3<f32>(p_trunc);
    // if sample_result.a >= 0.5 {
    //     output.hit = true;
    //     output.color = fract;
    // }
    output.alpha = sample_result.a;
    if output.alpha <= 0.5 {
        output.alpha = 0.0;
    }
    output.alpha = output.alpha / 1000.0;

    output.hit = true;
    output.color = fract;
    return output;
}

fn get_1d_index(p: vec3<i32>) -> i32{
    return p.x + i32(voxel_grid.dimensions.x) * (p.y + i32(voxel_grid.dimensions.y) * p.z);
}

fn get_voxel_alpha(voxel: Voxel) -> u32 {
    return voxel.color & 0xFF;
}