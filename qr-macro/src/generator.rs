use qr::LogoPixel;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QrGeneratorError {
    #[error("Qr generator error: {0}")]
    Qr(#[from] qrcode::types::QrError),
    #[error("Qr image error: {0}")]
    Image(#[from] image::error::ImageError),
    #[error("Syntax error")]
    Syn(#[from] syn::Error),
}

pub struct QrLogo {
    pub code: Vec<bool>,
    pub width: usize,
    pub logo_offset: [usize; 2],
    pub logo_width: usize,
    pub logo_inverts: Vec<bool>,
}

pub fn generate_qr_code_with_logo(
    vcard: &str,
    logo: impl ExactSizeIterator<Item = impl ExactSizeIterator<Item = LogoPixel>>,
    save_location: Option<&Path>,
) -> Result<QrLogo, QrGeneratorError> {
    let ec_level = EcLevel::Q;
    use image::Luma;
    use qrcode::{Color, EcLevel, canvas::Module};
    let bits = qrcode::bits::encode_auto(vcard.as_bytes(), ec_level)?;
    let version = bits.version();
    let data = bits.into_bytes();
    let (encoded_data, ec_data) = qrcode::ec::construct_codewords(&data, version, ec_level)?;
    let mut canvas = qrcode::canvas::Canvas::new(version, ec_level);
    canvas.draw_all_functional_patterns();
    canvas.draw_data(&encoded_data, &ec_data);
    let canvas = canvas.apply_best_mask();
    let mut new_canvas = canvas.clone();
    let width = version.width();
    let fi = |x| i16::try_from(x).unwrap();
    let mut logo = logo.peekable();
    if let Some(logo_first) = logo.peek() {
        let sx: i16 = (width - fi(logo_first.len())) / 2;
        let sy: i16 = (width - fi(logo.len())) / 2;
        for (y, line) in logo.enumerate() {
            let cy = sy + fi(y);
            for (x, c) in line.enumerate() {
                let cx = sx + fi(x);
                match c {
                    LogoPixel::Dark => *new_canvas.get_mut(cx, cy) = Module::Masked(Color::Dark),
                    LogoPixel::Light => *new_canvas.get_mut(cx, cy) = Module::Masked(Color::Light),
                    LogoPixel::Transparent => (),
                }
            }
        }
    }
    let original_colors = {
        let canvas = canvas.clone();
        canvas.into_colors()
    };
    let uwidth = width.try_into().unwrap();
    let new_colors = {
        let colors = new_canvas.into_colors();
        let mut renderer =
            qrcode::render::Renderer::<Luma<u8>>::new(&colors, width.try_into().unwrap(), 0);
        renderer.module_dimensions(1, 1);

        if let Some(location) = save_location {
            let image = renderer.build();
            image.save(location.join("qrcode.png"))?;
        }
        colors
    };
    let diff: Vec<Vec<bool>> = original_colors
        .chunks(uwidth)
        .zip(new_colors.chunks(uwidth))
        .map(|(orig, new)| {
            orig.iter()
                .zip(new.iter())
                .map(|(x, y)| x != y)
                .collect::<Vec<_>>()
        })
        .collect();
    let ranges = diff.iter().enumerate().fold(
        ((None, None), (None, None)),
        |((mut first_x, mut last_x), (mut first_y, mut last_y)), (line_idx, line)| {
            let mut has_match = false;
            line.iter().enumerate().for_each(|(idx, b)| {
                if *b {
                    has_match = true;
                    match first_x {
                        None => first_x = Some(idx),
                        Some(x) if idx < x => first_x = Some(idx),
                        Some(_) => (),
                    }
                    match last_x {
                        None => last_x = Some(idx),
                        Some(x) if idx > x => last_x = Some(idx),
                        Some(_) => (),
                    }
                }
            });
            if has_match {
                match first_y {
                    None => first_y = Some(line_idx),
                    Some(x) if line_idx < x => first_y = Some(line_idx),
                    Some(_) => (),
                }
                match last_y {
                    None => last_y = Some(line_idx),
                    Some(x) if line_idx > x => last_y = Some(line_idx),
                    Some(_) => (),
                }
            }
            ((first_x, last_x), (first_y, last_y))
        },
    );
    let mut result = QrLogo {
        code: original_colors.iter().map(|x| *x == Color::Light).collect(),
        logo_offset: [0, 0],
        width: uwidth,
        logo_width: uwidth,
        logo_inverts: vec![],
    };
    if let ((Some(x), Some(w)), (Some(y), Some(h))) = ranges {
        result.logo_offset = [x, y];
        result.logo_width = w - x + 1;
        result.logo_inverts = diff
            .into_iter()
            .skip(y)
            .take(h - y + 1)
            .flat_map(|line| line.into_iter().skip(x).take(w - x + 1))
            .collect();
    }
    Ok(result)
}
