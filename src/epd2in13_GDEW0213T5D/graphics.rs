use crate::epd2in13_GDEW0213T5D::{DEFAULT_BACKGROUND_COLOR, FRAME_BUFFER_SIZE, HEIGHT, WIDTH};
use crate::graphics::{Display, DisplayRotation};
use embedded_graphics_core::pixelcolor::BinaryColor;
use embedded_graphics_core::prelude::*;

/// Full size buffer for use with the 4in2 EPD
///
/// Can also be manuall constructed:
/// `buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); WIDTH / 8 * HEIGHT]`
pub struct DisplayT5D {
    // buffer: [u8; WIDTH as usize * HEIGHT as usize / 8],
    buffer: [u8; FRAME_BUFFER_SIZE],
    rotation: DisplayRotation,
}

impl Default for DisplayT5D {
    fn default() -> Self {
        DisplayT5D {
            buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); FRAME_BUFFER_SIZE],
            rotation: DisplayRotation::default(),
        }
    }
}

impl DrawTarget for DisplayT5D {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            self.draw_helper(WIDTH, HEIGHT, pixel)?;
        }
        Ok(())
    }
}

impl OriginDimensions for DisplayT5D {
    fn size(&self) -> Size {
        Size::new(WIDTH, HEIGHT)
    }
}

impl Display for DisplayT5D {
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    fn get_mut_buffer(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    fn set_rotation(&mut self, rotation: DisplayRotation) {
        self.rotation = rotation;
    }

    fn rotation(&self) -> DisplayRotation {
        self.rotation
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::color::Color;
//     use crate::epd2in13_GDEW0213T5D;
//     use crate::graphics::{Display, DisplayRotation};
//     use crate::{color::Black, epd2in13_GDEW0213T5D::Epd2in13_T5D};
//     use embedded_graphics::{
//         prelude::*,
//         primitives::{Line, PrimitiveStyle},
//     };

//     // test buffer length
//     #[test]
//     fn graphics_size() {
//         let display = DisplayT5D::default();
//         assert_eq!(display.buffer().len(), 2756);
//     }

//     // test default background color on all bytes
//     #[test]
//     fn graphics_default() {
//         let display = DisplayT5D::default();
//         for &byte in display.buffer() {
//             assert_eq!(
//                 byte,
//                 epd2in13_GDEW0213T5D::DEFAULT_BACKGROUND_COLOR.get_byte_value()
//             );
//         }
//     }

//     #[test]
//     fn graphics_rotation_0() {
//         let mut display = DisplayT5D::default();
//         let _ = Line::new(Point::new(0, 0), Point::new(7, 0))
//             .into_styled(PrimitiveStyle::with_stroke(Black, 1))
//             .draw(&mut display);

//         let buffer = display.buffer();

//         assert_eq!(buffer[0], Color::Black.get_byte_value());

//         for &byte in buffer.iter().skip(1) {
//             assert_eq!(
//                 byte,
//                 epd2in13_GDEW0213T5D::DEFAULT_BACKGROUND_COLOR.get_byte_value()
//             );
//         }
//     }

//     #[test]
//     fn graphics_rotation_90() {
//         let mut display = DisplayT5D::default();
//         display.set_rotation(DisplayRotation::Rotate90);
//         let _ = Line::new(Point::new(0, 392), Point::new(0, 399))
//             .into_styled(PrimitiveStyle::with_stroke(Black, 1))
//             .draw(&mut display);

//         let buffer = display.buffer();

//         assert_eq!(buffer[0], Color::Black.get_byte_value());

//         for &byte in buffer.iter().skip(1) {
//             assert_eq!(
//                 byte,
//                 epd2in13_GDEW0213T5D::DEFAULT_BACKGROUND_COLOR.get_byte_value()
//             );
//         }
//     }

//     #[test]
//     fn graphics_rotation_180() {
//         let mut display = DisplayT5D::default();
//         display.set_rotation(DisplayRotation::Rotate180);

//         let _ = Line::new(Point::new(392, 299), Point::new(399, 299))
//             .into_styled(PrimitiveStyle::with_stroke(Black, 1))
//             .draw(&mut display);

//         let buffer = display.buffer();

//         extern crate std;
//         std::println!("{:?}", buffer);

//         assert_eq!(buffer[0], Color::Black.get_byte_value());

//         for &byte in buffer.iter().skip(1) {
//             assert_eq!(
//                 byte,
//                 epd2in13_GDEW0213T5D::DEFAULT_BACKGROUND_COLOR.get_byte_value()
//             );
//         }
//     }

//     #[test]
//     fn graphics_rotation_270() {
//         let mut display = DisplayT5D::default();
//         display.set_rotation(DisplayRotation::Rotate270);
//         let _ = Line::new(Point::new(299, 0), Point::new(299, 7))
//             .into_styled(PrimitiveStyle::with_stroke(Black, 1))
//             .draw(&mut display);

//         let buffer = display.buffer();

//         extern crate std;
//         std::println!("{:?}", buffer);

//         assert_eq!(buffer[0], Color::Black.get_byte_value());

//         for &byte in buffer.iter().skip(1) {
//             assert_eq!(
//                 byte,
//                 epd2in13_GDEW0213T5D::DEFAULT_BACKGROUND_COLOR.get_byte_value()
//             );
//         }
//     }
// }
