use std::iter;

type QrCode<const WIDTH: usize, const HEIGHT: usize> = [[bool; WIDTH]; HEIGHT];

pub struct QrCodeLogoInverts<const WIDTH: usize, const HEIGHT: usize> {
    pub offset: [usize; 2],
    pub inverts: [[bool; WIDTH]; HEIGHT],
}

pub enum LogoPixel {
    Light,
    Dark,
    Transparent,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct QrCodeWithLogo<
    const W: usize,
    const H: usize,
    const X: usize,
    const Y: usize,
    const LW: usize,
    const LH: usize,
> {
    pub code: QrCode<W, H>,
    pub inverted: [[bool; LW]; LH],
}

impl<
    const W: usize,
    const H: usize,
    const X: usize,
    const Y: usize,
    const LW: usize,
    const LH: usize,
> QrCodeWithLogo<W, H, X, Y, LW, LH>
{
    pub fn iter_merged(&self) -> impl Iterator<Item = (bool, bool)> {
        let top_pad = iter::repeat_n(iter::repeat_n(false, W), Y);
        let left_pad = X;
        let right_pad = W - LW - X;
        let padded = self.inverted.iter().map(move |x| {
            iter::repeat_n(false, left_pad)
                .clone()
                .chain(x.into_iter().cloned())
                .chain(iter::repeat_n(false, right_pad))
        });
        let bottom_pad = iter::repeat_n(iter::repeat_n(false, W), H - LH - Y);
        let mask = top_pad
            .flatten()
            .chain(padded.flatten())
            .chain(bottom_pad.flatten());
        self.code.iter().flatten().cloned().zip(mask)
    }

    pub fn width(&self) -> usize {
        W
    }

    pub fn height(&self) -> usize {
        H
    }
}
