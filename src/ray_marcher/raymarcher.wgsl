struct CameraUniform {
    position: vec4<f32>,
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>
}

struct Voxel {
    color: vec3<f32>,
    alpha: f32
}

struct VoxelGrid {
    dimensions: vec3<u32>,
    buffer: u32
}

struct HitInfo {
    hit: bool,
    distance: f32,
    color: vec3<f32>
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

    if raymarch_result.hit {
        return vec4<f32>(raymarch_result.color, 1.0);
    } else {
        return vec4<f32>(vec3<f32>(raymarch_result.min_distance_to_scene), 1.0);
    }
}

fn raymarch(ro: vec3<f32>, rd: vec3<f32>) -> RayMarchOutput {
    var dt = 0.0;
    var output: RayMarchOutput = RayMarchOutput();

    output.min_distance_to_scene = 10000.0;
        for(var i = 0; i < 1000; i += 1) {
        let p: vec3<f32> = ro + rd * dt;
        let hitInfo = scene(p);

        if hitInfo.hit {
            output.hit = true;
            output.color = hitInfo.color;
            output.distance = dt;
            output.min_distance_to_scene = 0.0;
            return output;
        }

        dt += hitInfo.distance;
        output.steps = output.steps + 1;
    }
    output.distance = dt;
    return output;
}

fn scene(p: vec3<f32>) -> HitInfo {
    var output: HitInfo = HitInfo();
    let dimensions = voxel_grid.dimensions;
    let p_trunc: vec3<i32> = vec3<i32>(trunc(p));
    let fract: vec3<f32> = fract(p);
    let pos: vec3<i32> = p_trunc;
    let index = get_1d_index(pos);
    output.distance = 0.1;


    if p.x < 0.0 || p.y < 0.0 || p.z < 0.0 {
        return output;
    }
    if pos.x >= i32(dimensions.x) || pos.y >= i32(dimensions.y) || pos.z >= i32(dimensions.z) {
        return output;
    }

    let voxel = voxels[index];
    var relative_color = vec3<f32>(p_trunc);
    if voxel.alpha >= 0.5 {
        output.hit = true;
        output.distance = -1.0;
        output.color = voxel.color;
    }
    return output;
}

fn get_1d_index(p: vec3<i32>) -> i32{
    return p.x + i32(voxel_grid.dimensions.x) * (p.y + i32(voxel_grid.dimensions.y) * p.z);
}