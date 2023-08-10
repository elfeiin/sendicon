use jpegxl_rs::encoder_builder;

fn decode(input: Bytes) -> Image {
    image::load_from_memory(input)
}

pub fn convert(input: Bytes) -> image::Image {
    let img = decode(input);
}
