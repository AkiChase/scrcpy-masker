use ffmpeg_next::{Packet, codec};

pub enum VideoMsg {
    Start {
        codec_id: codec::Id,
        width: u32,
        height: u32,
    },
    Packet(Packet),
    End,
}
