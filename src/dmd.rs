use std::ffi::{c_char, CString};

use log::warn;
use sdl2::{pixels, rect::Rect};

use crate::libpinmame::{PinmameDisplayLayout, PinmameMechInfo, PINMAME_DISPLAY_TYPE_PINMAME_DISPLAY_TYPE_SEG16S};

const PIXEL_SIZE: u32 = 3;

// const PIXELS_WIDTH: u32 = 128;
// const PIXELS_HEIGHT: u32 = 32;

pub fn dmd_width(display_layout: &PinmameDisplayLayout) -> u32 {
    display_layout.width as u32 * (PIXEL_SIZE + 1)
}

pub fn dmd_height(display_layout: &PinmameDisplayLayout) -> u32 {
    display_layout.height as u32 * (PIXEL_SIZE + 1)
}

pub fn render_dmd(
    at_x: u32,
    at_y: u32,
    display_data: &[u8],
    display_layout: &PinmameDisplayLayout,
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
) -> Result<(), String> {
    for x in 0..display_layout.width as u32 {
        for y in 0..display_layout.height as u32 {
            // color from display_data
            let index = (y * display_layout.width as u32 + x) as usize;
            let value = display_data[index];

            let color = if display_layout.depth == 2 {
                match value {
                    0 => pixels::Color::RGB(40, 20, 0),
                    1 => pixels::Color::RGB(120, 80, 0),
                    2 => pixels::Color::RGB(215, 130, 0),
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
                (at_x + x * (PIXEL_SIZE + 1)) as i32,
                (at_y + y * (PIXEL_SIZE + 1)) as i32,
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

            let segments = if (*display_layout).type_ == PINMAME_DISPLAY_TYPE_PINMAME_DISPLAY_TYPE_SEG16S {
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

            warn!("TODO: dump alphanumeric not implemented yet");
            //unimplemented!("alphanumeric dmd dump not implemented yet")
        }
    }
}

const MECH_BAR_MULTIPLIER: u32 = 2;
pub fn render_mechs(
    at_x: u32,
    at_y: u32,
    mech_info: &[PinmameMechInfo],
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
) -> Result<(), String> {
    let mech_bar_height = 10;
    for (pos, mech) in mech_info.iter().enumerate() {
        let color = pixels::Color::RGB(150, 250, 150);
        let fill_color = pixels::Color::RGB(50, 150, 50);
        canvas.set_draw_color(fill_color);
        canvas.fill_rect(Rect::new(
            at_x as i32,
            at_y as i32 + pos as i32 * mech_bar_height as i32,
            mech.pos as u32 * MECH_BAR_MULTIPLIER,
            mech_bar_height,
        ))?;
        canvas.set_draw_color(color);
        canvas.draw_rect(Rect::new(
            at_x as i32,
            at_y as i32 + pos as i32 * mech_bar_height as i32,
            mech.length as u32 * MECH_BAR_MULTIPLIER,
            mech_bar_height,
        ))?;
    }
    Ok(())
}

pub fn render_lights(
    at_x: u32,
    at_y: u32,
    lamps: &[bool],
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    lamp_size: u32,
) -> Result<(), String> {
    // lights are numberedred from 1
    let mut lamp_index = 1;
    for x in 0..20 {
        for y in 0..10 {
            if lamp_index >= lamps.len() {
                break;
            }
            let color = if lamps[lamp_index] {
                pixels::Color::RGB(255, 255, 100)
            } else {
                pixels::Color::RGB(20, 20, 10)
            };
            canvas.set_draw_color(color);
            canvas.fill_rect(Rect::new(
                (at_x + x * (lamp_size + 1)) as i32,
                (at_y + y * (lamp_size + 1)) as i32,
                lamp_size,
                lamp_size,
            ))?;
            lamp_index += 1;
        }
    }
    Ok(())
}

pub(crate) fn render_solenoids(
    at_x: u32,
    at_y: u32,
    solenoids: &[bool],
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    solenoid_size: u32,
) -> Result<(), String> {
    // solenoids are numberedred from 1
    let mut solenoid_index = 1;
    for x in 0..20 {
        for y in 0..10 {
            if solenoid_index >= solenoids.len() {
                break;
            }

            let color = if solenoids[solenoid_index] {
                pixels::Color::RGB(255, 255, 100)
            } else {
                pixels::Color::RGB(20, 20, 10)
            };
            canvas.set_draw_color(color);
            canvas.fill_rect(Rect::new(
                (at_x + x * (solenoid_size + 1)) as i32,
                (at_y + y * (solenoid_size + 1)) as i32,
                solenoid_size,
                solenoid_size,
            ))?;
            solenoid_index += 1;
        }
    }
    Ok(())
}
