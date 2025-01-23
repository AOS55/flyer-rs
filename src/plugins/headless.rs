use bevy::{
    app::ScheduleRunnerPlugin,
    image::TextureFormatPixelInfo,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_asset::{RenderAssetUsages, RenderAssets},
        render_graph::{self, NodeRunError, RenderGraph, RenderGraphContext, RenderLabel},
        render_resource::{
            Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d,
            ImageCopyBuffer, ImageDataLayout, Maintain, MapMode, TextureDimension, TextureFormat,
            TextureUsages,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        Extract, Render, RenderApp, RenderSet,
    },
};
use crossbeam_channel::{Receiver, Sender};
use nalgebra::Vector3;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use crate::{
    components::{CameraComponent, PlayerController},
    plugins::{LatestFrame, StartupStage},
    resources::TransformationResource,
};

pub struct HeadlessPlugin {
    width: u32,
    height: u32,
    asset_path: Option<String>,
}

impl HeadlessPlugin {
    pub fn new(width: u32, height: u32, asset_path: String) -> Self {
        HeadlessPlugin {
            width,
            height,
            asset_path: Some(asset_path),
        }
    }

    pub fn new_no_path(width: u32, height: u32) -> Self {
        HeadlessPlugin {
            width,
            height,
            asset_path: None,
        }
    }
}

impl Plugin for HeadlessPlugin {
    fn build(&self, app: &mut App) {
        let asset_plugin = if let Some(asset_path) = &self.asset_path {
            AssetPlugin {
                file_path: asset_path.clone(),
                ..Default::default()
            }
        } else {
            AssetPlugin::default()
        };

        app.insert_resource(SceneController::new(self.width, self.height))
            .insert_resource(LatestFrame::new(self.width, self.height))
            .insert_resource(ClearColor(Color::srgb_u8(0, 0, 0)))
            .add_plugins(
                DefaultPlugins
                    .set(ImagePlugin::default_nearest())
                    .set(WindowPlugin {
                        primary_window: None,
                        exit_condition: bevy::window::ExitCondition::DontExit,
                        close_when_requested: false,
                    })
                    .set(asset_plugin),
            )
            .add_plugins(ImageCopyPlugin)
            .add_plugins(CaptureFramePlugin)
            .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0,
            )))
            .init_resource::<SceneController>()
            .add_systems(Startup, setup.in_set(StartupStage::BuildCameras));
    }
}

fn setup(
    mut commands: Commands,
    player_query: Query<Entity, With<PlayerController>>,
    mut images: ResMut<Assets<Image>>,
    mut scene_controller: ResMut<SceneController>,
    render_device: Res<RenderDevice>,
    transform_res: Res<TransformationResource>,
) {
    let render_target = setup_render_target(
        &mut commands,
        &mut images,
        &render_device,
        &mut scene_controller,
        40,
    );

    commands.insert_resource(ClearColor(Color::srgb(0.5, 0.5, 1.0)));

    // TODO: Problem is here, no camera spawned
    if let Ok(player_entity) = player_query.get_single() {
        let initial_pos = Vector3::new(0.0, 0.0, 500.0);
        let render_pos = transform_res.screen_from_ned(&initial_pos).unwrap();
        commands.spawn((
            Camera2d::default(),
            Camera {
                hdr: true,
                target: render_target.clone(),
                ..Default::default()
            },
            Transform::from_translation(render_pos), // Set the camera position in world space
            GlobalTransform::default(),
            CameraComponent {
                target: Some(player_entity),
                ..default()
            },
        ));
    } else {
        error!("No player entity found");
    }
}

#[derive(Resource, Deref)]
struct MainWorldReceiver(Receiver<Vec<u8>>);

#[derive(Resource, Deref)]
struct RenderWorldSender(Sender<Vec<u8>>);

#[derive(Debug, Default, Resource)]
struct SceneController {
    state: SceneState,
    width: u32,
    height: u32,
}

impl SceneController {
    pub fn new(width: u32, height: u32) -> SceneController {
        SceneController {
            state: SceneState::BuildScene,
            width,
            height,
        }
    }
}

#[derive(Debug, Default)]
enum SceneState {
    #[default]
    BuildScene,
    Render(u32),
}

pub struct ImageCopyPlugin;
impl Plugin for ImageCopyPlugin {
    fn build(&self, app: &mut App) {
        let (s, r) = crossbeam_channel::unbounded();

        let render_app = app
            .insert_resource(MainWorldReceiver(r))
            .sub_app_mut(RenderApp);

        let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
        graph.add_node(ImageCopy, ImageCopyDriver);
        graph.add_node_edge(bevy::render::graph::CameraDriverLabel, ImageCopy);

        render_app
            .insert_resource(RenderWorldSender(s))
            .add_systems(ExtractSchedule, image_copy_extract)
            .add_systems(Render, receive_image_from_buffer.after(RenderSet::Render));
    }
}

fn setup_render_target(
    commands: &mut Commands,
    images: &mut ResMut<Assets<Image>>,
    render_device: &Res<RenderDevice>,
    scene_controller: &mut ResMut<SceneController>,
    pre_roll_frames: u32,
) -> RenderTarget {
    let size = Extent3d {
        width: scene_controller.width,
        height: scene_controller.height,
        ..Default::default()
    };

    // This is the texture that will be rendered to.
    let mut render_target_image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0; 4],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    render_target_image.texture_descriptor.usage |=
        TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING;
    let render_target_image_handle = images.add(render_target_image);

    // This is the texture that will be copied to.
    let cpu_image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0; 4],
        TextureFormat::bevy_default(),
        RenderAssetUsages::default(),
    );
    let cpu_image_handle = images.add(cpu_image);

    commands.spawn(ImageCopier::new(
        render_target_image_handle.clone(),
        size,
        render_device,
    ));

    commands.spawn(ImageToSave(cpu_image_handle));

    scene_controller.state = SceneState::Render(pre_roll_frames);
    RenderTarget::Image(render_target_image_handle)
}

pub struct CaptureFramePlugin;
impl Plugin for CaptureFramePlugin {
    fn build(&self, app: &mut App) {
        info!("Adding CaptureFramePlugin");
        app.add_systems(PostUpdate, update);
    }
}

#[derive(Clone, Default, Resource, Deref, DerefMut)]
struct ImageCopiers(pub Vec<ImageCopier>);

#[derive(Clone, Component)]
struct ImageCopier {
    buffer: Buffer,
    enabled: Arc<AtomicBool>,
    src_image: Handle<Image>,
}

impl ImageCopier {
    pub fn new(
        src_image: Handle<Image>,
        size: Extent3d,
        render_device: &RenderDevice,
    ) -> ImageCopier {
        let padded_bytes_per_row =
            RenderDevice::align_copy_bytes_per_row((size.width) as usize) * 4;

        let cpu_buffer = render_device.create_buffer(&BufferDescriptor {
            label: None,
            size: padded_bytes_per_row as u64 * size.height as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        ImageCopier {
            buffer: cpu_buffer,
            src_image,
            enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
}

fn image_copy_extract(mut commands: Commands, image_copiers: Extract<Query<&ImageCopier>>) {
    commands.insert_resource(ImageCopiers(
        image_copiers.iter().cloned().collect::<Vec<ImageCopier>>(),
    ));
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, RenderLabel)]
struct ImageCopy;

#[derive(Default)]
struct ImageCopyDriver;

impl render_graph::Node for ImageCopyDriver {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        info!("ImageCopyDriver running");
        let image_copiers = world.get_resource::<ImageCopiers>().unwrap();
        let gpu_images = world
            .get_resource::<RenderAssets<bevy::render::texture::GpuImage>>()
            .unwrap();

        for image_copier in image_copiers.iter() {
            info!("Processing copier");
            if !image_copier.enabled() {
                continue;
            }

            let src_image = gpu_images.get(&image_copier.src_image).unwrap();

            let mut encoder = render_context
                .render_device()
                .create_command_encoder(&CommandEncoderDescriptor::default());

            let block_dimensions = src_image.texture_format.block_dimensions();
            let block_size = src_image.texture_format.block_copy_size(None).unwrap();

            let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(
                (src_image.size.x as usize / block_dimensions.0 as usize) * block_size as usize,
            );

            let texture_extent = Extent3d {
                width: src_image.size.x,
                height: src_image.size.y,
                depth_or_array_layers: 1,
            };

            encoder.copy_texture_to_buffer(
                src_image.texture.as_image_copy(),
                ImageCopyBuffer {
                    buffer: &image_copier.buffer,
                    layout: ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(
                            std::num::NonZero::<u32>::new(padded_bytes_per_row as u32)
                                .unwrap()
                                .into(),
                        ),
                        rows_per_image: None,
                    },
                },
                texture_extent,
            );

            let render_queue = world.get_resource::<RenderQueue>().unwrap();
            render_queue.submit(std::iter::once(encoder.finish()));
        }

        Ok(())
    }
}

fn receive_image_from_buffer(
    image_copiers: Res<ImageCopiers>,
    render_device: Res<RenderDevice>,
    sender: Res<RenderWorldSender>,
) {
    info!("Start buffer mapping");
    for image_copier in image_copiers.0.iter() {
        if !image_copier.enabled() {
            continue;
        }

        let buffer_slice = image_copier.buffer.slice(..);
        info!("Got buffer slice size: {}", buffer_slice.size());

        // Wait for buffer to be ready
        let (s, r) = crossbeam_channel::bounded(1);
        buffer_slice.map_async(MapMode::Read, move |result| {
            if let Ok(_) = result {
                s.send(()).expect("Failed to signal mapping complete");
            }
        });

        // Wait for mapping to complete before reading
        render_device.poll(Maintain::Wait).panic_on_timeout();
        if r.recv().is_ok() {
            let data = buffer_slice.get_mapped_range().to_vec();
            image_copier.buffer.unmap(); // Unmap before sending
            let _ = sender.send(data);
        }
    }
}

#[derive(Component, Deref, DerefMut)]
struct ImageToSave(Handle<Image>);

// Takes from channel image content sent from render world and saves it to disk
fn update(
    images_to_save: Query<&ImageToSave>,
    receiver: Res<MainWorldReceiver>,
    mut images: ResMut<Assets<Image>>,
    mut scene_controller: ResMut<SceneController>,
    mut latest_frame: ResMut<LatestFrame>,
) {
    if let SceneState::Render(n) = scene_controller.state {
        if n < 1 {
            // We don't want to block the main world on this,
            // so we use try_recv which attempts to receive without blocking
            let mut image_data = Vec::new();
            while let Ok(data) = receiver.try_recv() {
                // keep only the latest frame
                info!("Received image data: {} bytes", data.len());
                image_data = data;
            }
            if !image_data.is_empty() {
                info!("Processing image data: {} bytes", image_data.len());
                for image in images_to_save.iter() {
                    // Fill correct data from channel to image
                    let img_bytes = images.get_mut(image.id()).unwrap();

                    // Handle padding alignment
                    let row_bytes = img_bytes.width() as usize
                        * img_bytes.texture_descriptor.format.pixel_size();
                    let aligned_row_bytes = RenderDevice::align_copy_bytes_per_row(row_bytes);

                    let final_data = if row_bytes == aligned_row_bytes {
                        image_data.clone()
                    } else {
                        // Remove padding to get original image size
                        image_data
                            .chunks(aligned_row_bytes)
                            .take(img_bytes.height() as usize)
                            .flat_map(|row| &row[..row_bytes.min(row.len())])
                            .cloned()
                            .collect()
                    };

                    // Update the latest frame resource
                    latest_frame.update(final_data);
                    info!("Updated latest frame");
                }
            }
        } else {
            // clears channel for skipped frames
            while receiver.try_recv().is_ok() {}
            scene_controller.state = SceneState::Render(n - 1);
        }
    }
}
