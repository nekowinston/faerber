use super::RGB;
use super::{DitherResult, Error};
use std::ops::{Index, IndexMut};
use std::path::Path;
/// Image as a flat buffer of pixels; accessible by (x, y) [Index]
#[derive(Clone, Debug, PartialEq)]
pub struct Img<P> {
    buf: Vec<P>,
    width: u32,
}

impl<P> Img<P> {
    /// create an Img<P> from a buf and width. fails if `buf.len() % buf.width() != 0`
    pub fn new(buf: impl IntoIterator<Item = P>, width: u32) -> Option<Self> {
        let buf: Vec<P> = buf.into_iter().collect();
        if width == 0 || buf.len() % width as usize != 0 {
            None
        } else {
            Some(Img { buf, width })
        }
    }
    /// create an Img<P> from a buf and length directly, skipping the bounds check.
    /// ```
    /// # use dither::prelude::*;
    /// assert_eq!(
    ///     unsafe{Img::from_raw_buf(vec![2, 4, 6, 8], 2)},
    ///     Img::new(vec![2, 4, 6, 8], 2).unwrap()
    /// );
    /// ```
    pub const fn from_raw_buf(buf: Vec<P>, width: u32) -> Self {
        Img { buf, width }
    }

    /// pull the buffer out of the image as a vec.
    /// ```
    /// # use dither::prelude::*;
    /// assert_eq!(Img::new(1..=4, 2).unwrap().into_vec(), vec![1, 2, 3, 4]);
    /// ```
    pub fn into_vec(self) -> Vec<P> {
        self.buf
    }

    /// get the width of the image.
    /// ```
    /// # use dither::prelude::*;
    /// assert_eq!(Img::new(0..12, 3).unwrap().width(), 3);
    /// ```
    pub fn width(&self) -> u32 {
        self.width
    }
    /// returns an iterator over the pixels in the buffer
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }

    /// returns an iterator that allows modifying each pixel
    pub fn iter_mut(&mut self) -> <&mut Self as IntoIterator>::IntoIter {
        self.buf.iter_mut()
    }
    /// the height of the image; i.e, `buf.len() / width`
    /// ```
    /// # use dither::prelude::*;
    /// assert_eq!(Img::new(0..12, 3).unwrap().height(), 4);
    /// ```
    pub fn height(&self) -> u32 {
        self.len() as u32 / self.width
    }
    /// map a function on P across the image buffer, converting an `Img<P>` to an `Img<Q>`
    ///
    /// ```
    /// # use dither::prelude::*;
    /// let img: Img<u8> = Img::new(1..=4, 2).unwrap();
    /// let doubled: Img<u16> = Img::new(vec![2, 4, 6, 8], 2).unwrap();
    /// assert_eq!(img.convert_with(|x| u16::from(x*2)), doubled);
    /// ```
    pub fn convert_with<Q>(self, convert: impl Fn(P) -> Q) -> Img<Q> {
        let Img { buf, width } = self;
        Img {
            buf: buf.into_iter().map(convert).collect(),
            width,
        }
    }
    #[inline]
    fn idx(&self, (x, y): (u32, u32)) -> usize {
        ((y * self.width) + x) as usize
    }
    /// the length of the image, in _pixels_. equal to [Img::width()]*[Img::height()]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to an element.
    pub fn get(&self, (x, y): (u32, u32)) -> Option<&P> {
        self.buf.get(self.idx((x, y)))
    }
    /// Returns a pair `(width, height)`.
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.len() as u32 / self.width)
    }
}

impl<N: From<u8>> Img<RGB<N>> {
    /// load an image as an RGB<N> after converting it. See [image::open] and [image::DynamicImage::to_rgb]
    /// ```rust
    /// use dither::prelude::*;
    /// let img: Img<RGB<u8>> = Img::load("bunny.png").unwrap();
    /// assert_eq!(img.size(), (480, 320));
    /// ```
    pub fn load(path: impl AsRef<Path>) -> DitherResult<Self> {
        match image::open(&path).map(|img| img.to_rgb8()) {
            Err(err) => Err(Error::input(err, path.as_ref())),
            Ok(img) => Ok(Img {
                buf: img.pixels().map(|p| RGB::from(p.0)).collect(),
                width: img.width(),
            }),
        }
    }
}

impl Img<RGB<u8>> {
    /// save an image as a `.png` or `.jpg` to the path. the path extension determines the image type.
    /// See [image::ImageBuffer::save]
    pub fn save(self, path: &Path) -> DitherResult<()> {
        let (width, height) = self.size();
        let buf = image::RgbImage::from_raw(width, height, self.raw_buf()).unwrap();
        if let Err(err) = buf.save(path) {
            Err(Error::output(err, path))
        } else {
            Ok(())
        }
    }
    /// the raw_buf flattens out each RGB triplet;
    /// ```
    /// use dither::prelude::*;
    /// let img: Img<RGB<u8>> = Img::new(vec![RGB(0, 1, 2), RGB(1, 1, 1)], 1).unwrap();
    /// assert_eq!(img.raw_buf(), vec![0, 1, 2, 1, 1, 1]);
    /// ```
    pub fn raw_buf(self) -> Vec<u8> {
        let mut raw_buf = Vec::with_capacity(self.len() * 3);
        for RGB(r, g, b) in self.buf {
            raw_buf.push(r);
            raw_buf.push(g);
            raw_buf.push(b);
        }
        raw_buf
    }
}

impl<P> Index<(u32, u32)> for Img<P> {
    type Output = P;
    fn index(&self, (x, y): (u32, u32)) -> &P {
        &self.buf[self.idx((x, y))]
    }
}

impl<P> IndexMut<(u32, u32)> for Img<P> {
    fn index_mut(&mut self, (x, y): (u32, u32)) -> &mut P {
        let i = self.idx((x, y));
        &mut self.buf[i]
    }
}

impl<P> IntoIterator for Img<P> {
    type Item = P;
    type IntoIter = std::vec::IntoIter<P>;
    fn into_iter(self) -> Self::IntoIter {
        self.buf.into_iter()
    }
}

impl<'a, P> IntoIterator for &'a Img<P> {
    type Item = &'a P;
    type IntoIter = std::slice::Iter<'a, P>;
    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter()
    }
}

impl<'a, P> IntoIterator for &'a mut Img<P> {
    type Item = &'a mut P;
    type IntoIter = std::slice::IterMut<'a, P>;
    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter_mut()
    }
}
