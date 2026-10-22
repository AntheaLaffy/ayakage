use crate::{config::View, model::Model};
use anyhow::{Context, Result, bail};
use mocari::{
    assets::DecodedTexture,
    core::Matrix44,
    moc3::Moc3DrawableMesh,
    render::wgpu::{
        WgpuClippingPlan, WgpuClippingResources, WgpuLive2dRenderer, WgpuMaskRenderTarget,
        WgpuMeshBuffers, WgpuTexture, WgpuTransform,
    },
};

pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    min: [f32; 2],
    max: [f32; 2],
}
impl Bounds {
    pub fn from_meshes(meshes: &[Moc3DrawableMesh]) -> Result<Self> {
        let mut min = [f32::INFINITY; 2];
        let mut max = [f32::NEG_INFINITY; 2];
        // Hidden effect meshes can extend far beyond the character. Including
        // them makes the default camera frame the effects instead of the avatar.
        for vertex in meshes
            .iter()
            .filter(|mesh| mesh.opacity() > 0.001)
            .flat_map(Moc3DrawableMesh::vertices)
        {
            let p = vertex.position();
            for i in 0..2 {
                min[i] = min[i].min(p[i]);
                max[i] = max[i].max(p[i]);
            }
        }
        if !min.iter().chain(&max).all(|x| x.is_finite()) || max[0] <= min[0] || max[1] <= min[1] {
            bail!("模型没有有效的画布尺寸");
        }
        Ok(Self { min, max })
    }
    pub fn matrix(&self, size: [u32; 2], view: &View) -> Matrix44 {
        let aspect = size[0] as f32 / size[1].max(1) as f32;
        let width = self.max[0] - self.min[0];
        let height = self.max[1] - self.min[1];
        let scale = (1.85 / (width * aspect)).min(1.85 / height) * view.zoom.clamp(0.1, 6.0);
        let sx = scale / aspect;
        let mut matrix = Matrix44::identity();
        matrix.scale(sx, scale);
        matrix.translate(
            -(self.min[0] + self.max[0]) * 0.5 * sx + view.pan[0],
            -(self.min[1] + self.max[1]) * 0.5 * scale + view.pan[1],
        );
        matrix
    }
}

pub struct RenderTarget {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub size: [u32; 2],
}
impl RenderTarget {
    pub fn new(device: &wgpu::Device, size: [u32; 2]) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ayakage.model.target"),
            size: wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        Self {
            texture,
            view,
            size,
        }
    }
    pub fn save_png(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &std::path::Path,
    ) -> Result<()> {
        let bytes = self.read_pixels(device, queue)?;
        // GPU target contains premultiplied alpha. PNG stores straight alpha.
        let mut straight = bytes;
        for pixel in straight.as_chunks_mut::<4>().0 {
            if pixel[3] > 0 {
                for i in 0..3 {
                    pixel[i] =
                        ((pixel[i] as f32 * 255.0 / pixel[3] as f32).round() as u32).min(255) as u8;
                }
            }
        }
        image::save_buffer(
            path,
            &straight,
            self.size[0],
            self.size[1],
            image::ColorType::Rgba8,
        )?;
        Ok(())
    }
    pub fn read_pixels(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Vec<u8>> {
        let stride = (self.size[0] * 4).div_ceil(256) * 256;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ayakage.readback"),
            size: (stride * self.size[1]) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            self.texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(stride),
                    rows_per_image: None,
                },
            },
            self.texture.size(),
        );
        queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = tx.send(result);
            });
        device.poll(wgpu::PollType::wait_indefinitely())?;
        rx.recv()??;
        let mapped = buffer.slice(..).get_mapped_range()?;
        let mut bytes = Vec::with_capacity((self.size[0] * self.size[1] * 4) as usize);
        for row in mapped.chunks_exact(stride as usize) {
            bytes.extend_from_slice(&row[..self.size[0] as usize * 4]);
        }
        drop(mapped);
        buffer.unmap();
        Ok(bytes)
    }
}

pub struct Renderer {
    renderer: WgpuLive2dRenderer,
    buffers: WgpuMeshBuffers,
    textures: Vec<WgpuTexture>,
    clipping: WgpuClippingResources,
    mask: WgpuMaskRenderTarget,
    transform: WgpuTransform,
    bounds: Bounds,
}
impl Renderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, model: &Model) -> Result<Self> {
        let renderer = WgpuLive2dRenderer::new(device, FORMAT);
        let buffers = WgpuMeshBuffers::from_drawables(device, model.runtime().meshes())
            .context("无法创建模型顶点缓冲")?;
        let mut plan = WgpuClippingPlan::from_mesh_buffers(&buffers);
        plan.prepare_single_texture_masks(&buffers)?;
        let clipping = renderer.create_clipping_resources(device, &plan)?;
        let mask = renderer.create_mask_render_target(device, 512)?;
        let transform = renderer.create_transform(device, &Matrix44::identity());
        let bounds = Bounds::from_meshes(model.runtime().meshes())?;
        let textures = model
            .asset
            .textures()
            .iter()
            .map(|texture: &DecodedTexture| {
                renderer
                    .create_rgba8_texture(
                        device,
                        queue,
                        texture.width(),
                        texture.height(),
                        texture.rgba(),
                    )
                    .map_err(anyhow::Error::from)
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            renderer,
            buffers,
            textures,
            clipping,
            mask,
            transform,
            bounds,
        })
    }
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        model: &Model,
        target: &RenderTarget,
        view: &View,
    ) -> Result<()> {
        if self
            .buffers
            .update_drawables(queue, model.runtime().meshes())
            .is_err()
        {
            self.buffers = WgpuMeshBuffers::from_drawables(device, model.runtime().meshes())
                .context("模型缓冲更新失败")?;
        }
        let mut plan = WgpuClippingPlan::from_mesh_buffers(&self.buffers);
        plan.prepare_single_texture_masks(&self.buffers)?;
        if !self
            .renderer
            .update_clipping_resources(queue, &mut self.clipping, &plan)?
        {
            self.clipping = self.renderer.create_clipping_resources(device, &plan)?;
        }
        self.transform
            .update_matrix(queue, &self.bounds.matrix(target.size, view));
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ayakage.model.encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ayakage.mask"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: self.mask.view(),
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            self.renderer.draw_masks_with_textures(
                &mut pass,
                &self.buffers,
                &self.clipping,
                &self.textures,
            )?;
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ayakage.model"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            self.renderer.draw_with_textures_clipping_and_transform(
                &mut pass,
                &self.buffers,
                &self.textures,
                &self.clipping,
                &self.mask,
                &self.transform,
            )?;
        }
        queue.submit([encoder.finish()]);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fitting_preserves_aspect_and_pan_and_zoom() {
        let bounds = Bounds {
            min: [-2.0, -1.0],
            max: [2.0, 3.0],
        };
        let matrix = bounds.matrix([1920, 1080], &View::default());
        assert!((matrix.transform_x(0.0)).abs() < 0.0001);
        assert!((matrix.transform_y(1.0)).abs() < 0.0001);
        assert!((matrix.scale_x() * 1920.0 - matrix.scale_y() * 1080.0).abs() < 0.001);
        let view = View {
            zoom: 2.0,
            pan: [0.2, -0.4],
        };
        let moved = bounds.matrix([1920, 1080], &view);
        assert!((moved.scale_x() - matrix.scale_x() * 2.0).abs() < 0.0001);
        assert!((moved.transform_x(0.0) - 0.2).abs() < 0.0001);
        assert!((moved.transform_y(1.0) + 0.4).abs() < 0.0001);
    }
}
