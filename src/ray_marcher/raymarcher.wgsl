// Vertex shader
struct CameraUniform {
    position: vec4<f32>,
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>
}

struct Voxel {
    position: vec3<f32>,
    radius: f32
}

struct VoxelGrid {
    dimensions: vec3<u32>
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


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // return textureSample(t_diffuse, s_diffuse, in.tex_coords);
    var screen_position = vec4<f32>(in.tex_coords.x, in.tex_coords.y, 1.0, 1.0);
    screen_position -= vec4<f32>(0.5, 0.5, 0.0, 0.0);
    let world_position = camera.inverse_view_proj * screen_position;
    let dir: vec3<f32> = world_position.xyz;
    var ro: vec3<f32> = camera.position.xyz;

    let raymarch_result = raymarch(ro, dir);

    // if raymarch_result.hit {
    //     return vec4<f32>(raymarch_result.color, 1.0);
    // } else {
    //     return vec4<f32>(vec3<f32>(raymarch_result.min_distance_to_scene), 1.0);
    // }
    return vec4<f32>(vec3<f32>(f32(raymarch_result.steps) / 20.0), 1.0);
}

fn raymarch(ro: vec3<f32>, rd: vec3<f32>) -> RayMarchOutput {
    var dt = 0.0;
    var output: RayMarchOutput = RayMarchOutput();
    output.min_distance_to_scene = 10000.0;
        for(var i = 0; i < 20; i += 1) {
        let p: vec3<f32> = ro + rd * dt;
        let distance = scene(p);

        if distance < output.min_distance_to_scene {
            output.min_distance_to_scene = distance;
        }

        if distance < 0.01 {
            let normal = get_normal(p);
            let diffuse = max(dot(normal, normalize(vec3<f32>(1.0, 1.0, 0.0))), 0.0);
            output.hit = true;
            output.color = vec3<f32>(diffuse);
            output.distance = dt + distance;
            output.min_distance_to_scene = 0.0;
            return output;
        }

        dt += distance;
        output.steps = output.steps + 1;
    }
    output.distance = dt;
    return output;
}

fn scene(p: vec3<f32>) -> f32 {
    var distance = 10000.0;
    let length: u32 = arrayLength(&voxels);
    for(var i: u32 = 0; i < 20 * 20 * 20; i++) {
        // distance = min(sphere_distance(p - voxels[i].position, voxels[i].radius), distance);
        distance = min(sphere_distance(p, 1.0), distance);
    }
    return distance;
    // return sphere_distance(p, 1.0);
}

fn sphere_distance(ray_pos: vec3<f32>, radius: f32) -> f32 {
    return length(ray_pos) - radius;
}

fn get_normal(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.01, 0);

    let n = scene(p) - vec3<f32>(
        scene(p-e.xyy),
        scene(p-e.yxy),
        scene(p-e.yyx));
    
    return normalize(n);
}