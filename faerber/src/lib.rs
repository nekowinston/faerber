use faerber_lib::custom_lab::Lab;

pub fn get_labs(palette: Vec<u32>) -> Vec<Lab> {
    palette
        .iter()
        .map(|c| {
            Lab::from_rgb(&[
                ((c >> 16) & 0xFF) as u8,
                ((c >> 8) & 0xFF) as u8,
                (c & 0xFF) as u8,
            ])
        })
        .collect()
}
