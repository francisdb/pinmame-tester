use std::ffi::{c_char, CString};

use sdl2::{pixels, rect::Rect};

use crate::libpinmame::{PinmameDisplayLayout, PINMAME_DISPLAY_TYPE_SEG16S};

const PIXEL_SIZE: u32 = 4;

const PIXELS_WIDTH: u32 = 128;
const PIXELS_HEIGHT: u32 = 32;

pub const SCREEN_WIDTH: u32 = PIXELS_WIDTH * (PIXEL_SIZE + 1);
pub const SCREEN_HEIGHT: u32 = PIXELS_HEIGHT * (PIXEL_SIZE + 1);

pub fn render_dmd(
    display_data: &[u8],
    display_layout: &PinmameDisplayLayout,
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
) -> Result<(), String> {
    for x in 0..PIXELS_WIDTH {
        for y in 0..PIXELS_HEIGHT {
            // color from display_data
            let index = (y * PIXELS_WIDTH + x) as usize;
            let value = display_data[index];

            let color = if display_layout.depth == 2 {
                match value {
                    0 => pixels::Color::RGB(40, 20, 0),
                    1 => pixels::Color::RGB(180, 120, 0),
                    2 => pixels::Color::RGB(225, 140, 0),
                    3 => pixels::Color::RGB(250, 160, 0),
                    _ => unreachable!("depth 2 should only have 0-3"),
                }
            } else {
                match value {
                    0..=3 => pixels::Color::RGB(40, 20, 0),
                    4..=7 => pixels::Color::RGB(255, 140, 0),
                    8..=11 => pixels::Color::RGB(255, 152, 0),
                    12..=15 => pixels::Color::RGB(255, 165, 0),
                    _ => unreachable!("depth 4 should only have 0-15"),
                }
            };

            canvas.set_draw_color(color);
            canvas.fill_rect(Rect::new(
                (x * (PIXEL_SIZE + 1)) as i32,
                (y * (PIXEL_SIZE + 1)) as i32,
                PIXEL_SIZE,
                PIXEL_SIZE,
            ))?;
        }
    }
    Ok(())
}

// void DumpAlphanumeric(int index, UINT16* p_displayData, PinmameDisplayLayout* p_displayLayout) {
// 	char output[8][512] = {
// 		{ '\0' },
// 		{ '\0' },
// 		{ '\0' },
// 		{ '\0' },
// 		{ '\0' },
// 		{ '\0' },
// 		{ '\0' },
// 		{ '\0' }
// 	};

// 	for (int pos = 0; pos < p_displayLayout->length; pos++) {
// 		const UINT16 value = *(p_displayData++);

// 		char segments_16c[8][10] = {
// 			{ " AAAAA   " },
// 			{ "FI J KB  " },
// 			{ "F IJK B  " },
// 			{ " GG LL   " },
// 			{ "E ONM C  " },
// 			{ "EO N MC P" },
// 			{ " DDDDD  H" },
// 			{ "       H " },
// 		};

// 		char segments_16s[8][10] = {
// 			{ " AA BB   " },
// 			{ "HI J KC  " },
// 			{ "H IJK C  " },
// 			{ " PP LL   " },
// 			{ "G ONM D  " },
// 			{ "GO N MD  " },
// 			{ " FF EE   " },
// 			{ "         " },
// 		};

// 		char (*segments)[10] = (p_displayLayout->type == SEG16S) ? segments_16s : segments_16c;

// 		for (int row = 0; row < 8; row++) {
// 			for (int column = 0; column < 9; column++) {
// 				for (UINT16 bit = 0; bit < 16; bit++) {
// 					if (segments[row][column] == ('A' + bit)) {
// 						segments[row][column] = (value & (1 << bit)) ? '*' : ' ';
// 						break;
// 					}
// 				}
// 			}

// 			strcat(output[row], segments[row]);
// 			strcat(output[row], " ");
// 		}
// 	}

// 	for (int row = 0; row < 8; row++) {
// 		printf("%s\n", output[row]);
// 	}
// }

pub(crate) fn dump_alphanumeric(
    index: i32,
    display_data: *mut u16,
    display_layout: *mut PinmameDisplayLayout,
) {
    unsafe {
        let mut output: [[c_char; 512]; 8] = [[0; 512]; 8];

        for pos in 0..(*display_layout).length {
            let value = *display_data.offset(pos as isize);

            let segments_16c = [
                " AAAAA   ",
                "FI J KB  ",
                "F IJK B  ",
                " GG LL   ",
                "E ONM C  ",
                "EO N MC P",
                " DDDDD  H",
                "       H ",
            ];

            let segments_16s = [
                " AA BB   ",
                "HI J KC  ",
                "H IJK C  ",
                " PP LL   ",
                "G ONM D  ",
                "GO N MD  ",
                " FF EE   ",
                "         ",
            ];

            let segments = if (*display_layout).type_ == PINMAME_DISPLAY_TYPE_SEG16S {
                segments_16s
            } else {
                segments_16c
            };

            // 		for (int row = 0; row < 8; row++) {
            // 			for (int column = 0; column < 9; column++) {
            // 				for (UINT16 bit = 0; bit < 16; bit++) {
            // 					if (segments[row][column] == ('A' + bit)) {
            // 						segments[row][column] = (value & (1 << bit)) ? '*' : ' ';
            // 						break;
            // 					}
            // 				}
            // 			}

            // 			strcat(output[row], segments[row]);
            // 			strcat(output[row], " ");
            // 		}

            // same as above but in rust instead of c
            for row in 0..8 {
                for column in 0..9 {
                    for bit in 0..16 {
                        if segments[row].as_bytes()[column] == ('A' as u8 + bit as u8) {
                            // segments[row].as_bytes_mut()[column] =
                            //     if value & (1 << bit) != 0 { '*' } else { ' ' } as u8;
                            break;
                        }
                    }
                }

                let mut s = CString::new(segments[row]).unwrap();
                let s2 = CString::new(" ").unwrap();
                // let s3 = CString::new(output[row].as_mut_ptr()).unwrap();
                // s = s + s2;
                // s = s + s3;
                // output[row] = s.into_bytes_with_nul();
            }

            unimplemented!("alphanumeric dmd dump not implemented yet")
        }
    }
}