use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

use ffmpeg_next::{
    codec::{Context, flag::Flags},
    decoder,
    format::Pixel,
    frame,
    software::scaling,
};
use rust_i18n::t;

use crate::{
    mask::ui::basic::BORDER_THICKNESS, scrcpy::video_msg::VideoMsg, utils::ChannelReceiverV,
};

pub struct Scaler {
    width: u32,
    height: u32,
    scaler_context: scaling::Context,
    frame_id: Handle<Image>,
}

#[derive(Default)]
pub struct VideoResource {
    decoder: Option<decoder::Video>,
    scaler: Option<Scaler>,
}

impl VideoResource {
    fn create_image(
        width: u32,
        height: u32,
        images: &mut ResMut<Assets<Image>>,
        video_node: &mut Single<(&mut ImageNode, &mut VideoPlayer)>,
    ) -> Handle<Image> {
        let mut image = Image::new_fill(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        );
        image.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING;
        let image_handle = images.add(image);
        video_node.0.image = image_handle.clone();
        image_handle
    }

    pub fn create_scaler(
        &mut self,
        images: &mut ResMut<Assets<Image>>,
        video_node: &mut Single<(&mut ImageNode, &mut VideoPlayer)>,
    ) {
        let decoder = self.decoder.as_ref().unwrap();
        let width = decoder.width();
        let height = decoder.height();
        let scaler_context = scaling::Context::get(
            decoder.format(),
            width,
            height,
            Pixel::RGBA,
            width,
            height,
            scaling::Flags::BILINEAR,
        )
        .unwrap();

        self.scaler = Some(Scaler {
            width,
            height,
            scaler_context,
            frame_id: Self::create_image(width, height, images, video_node),
        });
    }

    pub fn update_scaler(
        &mut self,
        images: &mut ResMut<Assets<Image>>,
        video_node: &mut Single<(&mut ImageNode, &mut VideoPlayer)>,
    ) {
        let decoder = self.decoder.as_ref().unwrap();
        let scaler = self.scaler.as_mut().unwrap();
        let width = decoder.width();
        let height = decoder.height();
        if scaler.width != width || scaler.height != height {
            scaler.width = width;
            scaler.height = height;
            scaler.scaler_context = scaling::Context::get(
                decoder.format(),
                width,
                height,
                Pixel::RGBA,
                width,
                height,
                scaling::Flags::BILINEAR,
            )
            .unwrap();

            scaler.frame_id = Self::create_image(width, height, images, video_node);
        }
    }
}

#[derive(Component)]
pub struct VideoPlayer;

pub fn init_video(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            padding: UiRect::all(Val::Px(BORDER_THICKNESS)),
            box_sizing: BoxSizing::BorderBox,
            ..default()
        },
        ZIndex(-1),
        BackgroundColor(Color::NONE),
        ImageNode::default(),
        VideoPlayer,
    ));
}

pub fn handle_video_msg(
    v_rx: Res<ChannelReceiverV>,
    mut images: ResMut<Assets<Image>>,
    mut video_resource: NonSendMut<VideoResource>,
    mut video_node: Single<(&mut ImageNode, &mut VideoPlayer)>,
) {
    for msg in v_rx.0.try_iter() {
        match msg {
            VideoMsg::Start {
                codec_id,
                // width,
                // height,
                ..
            } => {
                let codec = decoder::find(codec_id).unwrap();
                let mut codec_context = Context::new_with_codec(codec);

                let flags = unsafe {
                    let raw_flags = (*codec_context.as_mut_ptr()).flags;
                    let flags =
                        Flags::from_bits(raw_flags as std::ffi::c_uint).unwrap_or(Flags::empty());
                    flags | Flags::LOW_DELAY
                };
                codec_context.set_flags(flags);
                let decoder = codec_context.decoder().video().unwrap();
                video_resource.decoder = Some(decoder);
                video_resource.scaler = None;
            }
            VideoMsg::Packet(mut packet) => {
                let decoded = {
                    let mut decoded = frame::Video::empty();
                    let decoder = video_resource
                        .decoder
                        .as_mut()
                        .expect(&t!("mask.video.noDecoder").to_string());
                    decoder.send_packet(&mut packet).unwrap();
                    decoder.receive_frame(&mut decoded).unwrap();
                    decoded
                };

                if video_resource.scaler.is_none() {
                    video_resource.create_scaler(&mut images, &mut video_node);
                } else {
                    video_resource.update_scaler(&mut images, &mut video_node);
                }

                let scaler = video_resource.scaler.as_mut().unwrap();

                if let Some(image) = images.get_mut(&scaler.frame_id) {
                    let mut rgb_frame = frame::Video::empty();
                    scaler.scaler_context.run(&decoded, &mut rgb_frame).unwrap();
                    image
                        .data
                        .as_mut()
                        .unwrap()
                        .copy_from_slice(rgb_frame.data(0));
                }
            }
            VideoMsg::End => {
                video_resource.decoder = None;
                video_resource.scaler = None;
            }
        }
    }
}
