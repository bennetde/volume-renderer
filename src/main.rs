use volume_renderer::run;

/// Entry
fn main() {
    pollster::block_on(run());
}
