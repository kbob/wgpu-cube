pub trait Renderable<Attributes, PreparedData> {
    fn prepare(&self, _: &Attributes) -> PreparedData;

    fn render<'rpass>(
        &'rpass self,
        _: &wgpu::Queue,
        _: &mut wgpu::RenderPass<'rpass>,
        _: &'rpass PreparedData,
    );
}
