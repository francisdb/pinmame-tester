use std::{
    ffi::{c_char, c_void, CString},
    sync::mpsc,
};

use log::{debug, error, info, warn};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::{
    audio::{AudioQueue, AudioSpecDesired},
    pixels,
};

use dmd::{SCREEN_HEIGHT, SCREEN_WIDTH};
use libpinmame::{
    va_list, PinmameAudioInfo, PinmameConfig, PinmameDisplayLayout, PinmameMechInfo,
    PINMAME_AUDIO_FORMAT_AUDIO_FORMAT_INT16, PINMAME_KEYCODE_ESCAPE, PINMAME_KEYCODE_MENU,
    PINMAME_KEYCODE_Q,
};
use pinmame::{Game, PinmameStatus};

use crate::pinmame::DmdMode;

mod dmd;
#[allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals
)]
mod libpinmame;
mod pinmame;

extern "C" fn pinmame_on_state_updated_callback(state: i32, _p_user_data: *const c_void) {
    info!("OnStateUpdated(): state={}", state);

    if state == 0 {
        std::process::exit(1);
    } else {
        // We had to come up with our own defaults
        // Are these correct?
        let mech_config = libpinmame::PinmameMechConfig {
            type_: (libpinmame::PINMAME_MECH_FLAGS_NONLINEAR
                | libpinmame::PINMAME_MECH_FLAGS_REVERSE
                | libpinmame::PINMAME_MECH_FLAGS_ONESOL) as i32,
            sol1: 11,
            sol2: 0,
            length: 240,
            steps: 240,
            initialPos: 0,
            acc: 0,
            ret: 0,
            sw: [libpinmame::PinmameMechSwitchConfig {
                swNo: 32,
                startPos: 0,
                endPos: 5,
                pulse: 0,
            }; 20],
        };

        unsafe {
            libpinmame::PinmameSetMech(0, &mech_config);
        }
    }
}

extern "C" fn pinmame_on_display_available_callback(
    index: i32,
    display_count: i32,
    display_layout: *mut libpinmame::PinmameDisplayLayout,
    _p_user_data: *const c_void,
) {
    unsafe {
        info!(
            "OnDisplayAvailable(): index={}, displayCount={}, type={}, top={}, left={}, width={}, height={}, depth={}, length={}",
            index,
            display_count,
            (*display_layout).type_,
            (*display_layout).top,
            (*display_layout).left,
            (*display_layout).width,
            (*display_layout).height,
            (*display_layout).depth,
            (*display_layout).length
        );
        // set the display layout
        let tester: &mut Tester = &mut *(_p_user_data as *mut Tester);
        tester.display_layout = Some(*display_layout);
    }
}

extern "C" fn pinmame_on_mech_available_callback(
    mech_no: ::std::os::raw::c_int,
    mech_info: *mut PinmameMechInfo,
    _user_data: *const ::std::os::raw::c_void,
) {
    unsafe {
        info!(
            "OnMechAvailable(): mechNo={}, type={}, length={}, steps={}, pos={}, speed={}",
            mech_no,
            (*mech_info).type_,
            (*mech_info).length,
            (*mech_info).steps,
            (*mech_info).pos,
            (*mech_info).speed
        );
    }
}

unsafe extern "C" fn pinmame_on_log_message_callback(
    log_level: u32,
    format: *const ::std::os::raw::c_char,
    args: va_list,
    _user_data: *const ::std::os::raw::c_void,
) {
    unsafe {
        let str = vsprintf::vsprintf(format, args).unwrap();
        match log_level {
            libpinmame::PINMAME_LOG_LEVEL_LOG_DEBUG => {
                debug!(target: "pinmame", "{}", str);
            }
            libpinmame::PINMAME_LOG_LEVEL_LOG_INFO => {
                info!(target: "pinmame", "{}", str);
            }
            libpinmame::PINMAME_LOG_LEVEL_LOG_ERROR => {
                error!(target: "pinmame", "{}", str);
            }
            _ => warn!("Unknown log level: {}", log_level),
        }
    }
}

unsafe extern "C" fn pinmame_on_console_data_updated_callback(
    _data: *mut ::std::os::raw::c_void,
    size: i32,
    _user_data: *const ::std::os::raw::c_void,
) {
    info!("OnConsoleDataUpdated: size={}", size);
}

unsafe extern "C" fn pinmame_on_display_updated_callback(
    index: i32,
    _display_data: *mut ::std::os::raw::c_void,
    display_layout: *mut libpinmame::PinmameDisplayLayout,
    _user_data: *const ::std::os::raw::c_void,
) {
    debug!(
        "OnDisplayUpdated(): index={}, type={}, top={}, left={}, width={}, height={}, depth={}, length={}",
        index,
        (*display_layout).type_,
        (*display_layout).top,
        (*display_layout).left,
        (*display_layout).width,
        (*display_layout).height,
        (*display_layout).depth,
        (*display_layout).length
    );

    if !_display_data.is_null() {
        if ((*display_layout).type_ & libpinmame::PINMAME_DISPLAY_TYPE_DMD)
            == libpinmame::PINMAME_DISPLAY_TYPE_DMD
        {
            debug!("DMD");
            //dump_dmd(index, _display_data as *mut u8, display_layout);
            let tester: &mut Tester = unsafe { &mut *(_user_data as *mut Tester) };
            match tester.display_data.send(
                std::slice::from_raw_parts_mut(
                    _display_data as *mut u8,
                    (*display_layout).width as usize * (*display_layout).height as usize,
                )
                .to_owned(),
            ) {
                Ok(_) => {}
                Err(e) => {
                    error!("display_data send failed: {}", e);
                }
            }
        } else {
            //debug!("Alphanumeric");
            dmd::dump_alphanumeric(index, _display_data as *mut u16, display_layout);
        }
    }
}

unsafe extern "C" fn pinmame_on_audio_available_callback(
    audio_info: *mut libpinmame::PinmameAudioInfo,
    _user_data: *const ::std::os::raw::c_void,
) -> i32 {
    info!(
        "OnAudioAvailable(): format={}, channels={}, sampleRate={}, framesPerSecond={}, samplesPerFrame={}, bufferSize={}",
        (*audio_info).format,
        (*audio_info).channels,
        (*audio_info).sampleRate,
        (*audio_info).framesPerSecond,
        (*audio_info).samplesPerFrame,
        (*audio_info).bufferSize
    );

    let tester: &mut Tester = unsafe { &mut *(_user_data as *mut Tester) };
    tester.audio_info = Some(*audio_info);

    (*audio_info).samplesPerFrame
}

extern "C" fn pinmame_on_audio_updated_callback(
    _buffer: *mut ::std::os::raw::c_void,
    samples: i32,
    _user_data: *const c_void,
) -> i32 {
    // trace!("OnAudioUpdated(): samples={}", samples);

    let tester = unsafe { &*(_user_data as *const Tester) };

    let samples_buffer =
        unsafe { std::slice::from_raw_parts(_buffer as *const i16, samples as usize) };

    match tester.rom_audio_queue.queue_audio(samples_buffer) {
        Ok(_) => {}
        Err(e) => {
            error!("queue_audio failed: {}", e);
        }
    };

    samples
}

extern "C" fn pinmame_on_solenoid_updated_callback(
    solenoid_state: *mut libpinmame::PinmameSolenoidState,
    _user_data: *const ::std::os::raw::c_void,
) {
    unsafe {
        info!(
            "OnSolenoidUpdated: solenoid={}, state={}",
            (*solenoid_state).solNo,
            (*solenoid_state).state
        );
    }
}

unsafe extern "C" fn pinmame_on_mech_updated_callback(
    mech_no: i32,
    mech_info: *mut PinmameMechInfo,
    _user_data: *const ::std::os::raw::c_void,
) {
    info!(
        "OnMechUpdated: mechNo={}, type={}, length={}, steps={}, pos={}, speed={}",
        mech_no,
        (*mech_info).type_,
        (*mech_info).length,
        (*mech_info).steps,
        (*mech_info).pos,
        (*mech_info).speed
    );
}

extern "C" fn pinmame_is_key_pressed_callback(
    _keycode: libpinmame::PINMAME_KEYCODE,
    _user_data: *const ::std::os::raw::c_void,
) -> i32 {
    //info!("IsKeyPressed: keycode={}", _keycode);
    let tester = unsafe { &*(_user_data as *const Tester) };
    let pressed = tester.keyboard_state[_keycode as usize];
    // somehow we should have access to osd_get_key_list to show the key description?
    // if pressed {
    //     info!("IsKeyPressed: keycode={} pressed", _keycode);
    // }
    pressed as i32
}

struct Tester {
    // TODO make all this thread safe
    rom_audio_queue: AudioQueue<i16>,
    audio_info: Option<PinmameAudioInfo>,
    display_layout: Option<PinmameDisplayLayout>,
    display_data: mpsc::Sender<Vec<u8>>,
    keyboard_state: [bool; (PINMAME_KEYCODE_MENU + 1) as usize],
}

fn main() -> Result<(), String> {
    // run me like this: RUST_LOG=info cargo run

    // info in T2 here
    // https://github.com/VisualPinball/VisualPinball.Engine.PinMAME/blob/master/VisualPinball.Engine.PinMAME/Games/Terminator2.cs

    pretty_env_logger::init();

    // TODO we should get this from the loaded rom, or update the window when we get it
    let (sdl_context, mut canvas) = setup_sdl2(SCREEN_WIDTH, SCREEN_HEIGHT)?;
    let mut events = sdl_context.event_pump()?;
    let audio_subsystem = sdl_context.audio()?;

    let (dmd_tx, dmd_rx) = mpsc::channel::<Vec<u8>>();

    // TODO we need to get this from the rom
    let desired_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };

    // None: use default device
    let device = audio_subsystem.open_queue::<i16, _>(None, &desired_spec)?;
    device.resume();

    let mut tester = Tester {
        rom_audio_queue: device,
        audio_info: None,
        display_layout: None,
        display_data: dmd_tx,
        keyboard_state: [false; (PINMAME_KEYCODE_MENU + 1) as usize],
    };

    // get home directory
    let home = dirs::home_dir().unwrap();
    let pinmame_path = home.join(".pinmame/");
    info!("Using path: {}", pinmame_path.display());

    let path = CString::new(pinmame_path.to_str().unwrap()).unwrap();
    let mut vpm_path: [c_char; 512] = [0; 512];
    for (i, c) in path.as_bytes_with_nul().iter().enumerate() {
        vpm_path[i] = *c as c_char;
    }
    let config = PinmameConfig {
        audioFormat: PINMAME_AUDIO_FORMAT_AUDIO_FORMAT_INT16,
        sampleRate: 44100,
        vpmPath: vpm_path,
        cb_OnStateUpdated: Some(pinmame_on_state_updated_callback),
        cb_OnDisplayAvailable: Some(pinmame_on_display_available_callback),
        cb_OnDisplayUpdated: Some(pinmame_on_display_updated_callback),
        cb_OnAudioAvailable: Some(pinmame_on_audio_available_callback),
        cb_OnAudioUpdated: Some(pinmame_on_audio_updated_callback),
        cb_OnMechAvailable: Some(pinmame_on_mech_available_callback),
        cb_OnMechUpdated: Some(pinmame_on_mech_updated_callback),
        cb_OnSolenoidUpdated: Some(pinmame_on_solenoid_updated_callback),
        cb_OnConsoleDataUpdated: Some(pinmame_on_console_data_updated_callback),
        fn_IsKeyPressed: Some(pinmame_is_key_pressed_callback),
        cb_OnLogMessage: Some(pinmame_on_log_message_callback),
    };

    //PinmameRun("mm_109c");
    //PinmameRun("fh_906h");
    //PinmameRun("hh7");
    //PinmameRun("rescu911");
    //PinmameRun("tf_180h");
    //PinmameRun("flashgdn");
    //PinmameRun("fourx4");
    //PinmameRun("ripleys");
    //PinmameRun("fh_l9");
    //PinmameRun("acd_170hc");
    //PinmameRun("snspares");

    let terminator_2_game_name = "t2_l8";
    // let medieval_madness_game_name = "mm_109c";
    // let fourx4_game_name = "fourx4";

    pinmame::set_config(&config);

    pinmame::set_user_data(&tester as *const Tester as *const std::ffi::c_void);
    pinmame::set_handle_keyboard(true);
    pinmame::set_handle_mechanics(true);

    pinmame::set_dmd_mode(DmdMode::Raw);
    match pinmame::get_games() {
        Ok(games) => {
            info!("Found {} games", games.len());
            let mut games = games;
            games.sort_by(|a, b| a.name.cmp(&b.name));
            for game in games {
                info!("  {}", describe_game(game));
            }
        }
        Err(status) => {
            error!("get_games() failed: {:?}", status);
        }
    }

    let p_name = terminator_2_game_name;

    match pinmame::get_game(p_name) {
        Ok(game) => {
            info!("Found game for {}: {}", p_name, describe_game(game));
        }
        Err(status) => {
            error!("Could not find game {}: {:?}", p_name, status);
        }
    }

    if pinmame::run(p_name) == PinmameStatus::Ok {
        info!("PinmameRun succeeded")
    } else {
        error!("PinmameRun failed");
    }

    let max_lamps = pinmame::get_max_lamps();
    info!("max_lamps: {}", max_lamps);
    let max_solenoids = pinmame::get_max_solenoids();
    info!("max_solenoids: {}", max_solenoids);

    'main: loop {
        // get the inputs here
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,

                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(keycode) = map_keycode(keycode) {
                        tester.keyboard_state[keycode as usize] = true;
                    }
                    match keycode {
                        Keycode::Escape => {
                            tester.keyboard_state[PINMAME_KEYCODE_ESCAPE as usize] = true;
                            break 'main;
                        }
                        Keycode::Q => {
                            tester.keyboard_state[PINMAME_KEYCODE_Q as usize] = true;
                            break 'main;
                        }
                        _ => {}
                    }
                    //println!("key down: {:?}", keycode);
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(keycode) = map_keycode(keycode.unwrap()) {
                        tester.keyboard_state[keycode as usize] = false;
                    }
                    //println!("key up: {:?}", keycode);
                }

                Event::MouseButtonDown { x, y, .. } => {
                    // canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
                    // canvas.clear();
                    // let color = pixels::Color::RGB(x as u8, y as u8, 255);
                    // let _ = canvas.line(lastx, lasty, x as i16, y as i16, color);
                    // lastx = x as i16;
                    // lasty = y as i16;
                    println!("mouse btn down at ({},{})", x, y);
                    // canvas.present();
                }

                Event::MouseButtonUp {
                    // timestamp,
                    // window_id,
                    // which,
                    // mouse_btn,
                    // clicks,
                    x,
                    y,
                    ..
                } => {
                    println!("mouse btn up at ({},{})", x, y);
                }

                _ => {}
            }
        }

        // update the game loop here

        let changed_lamps = pinmame::get_changed_lamps();
        if !changed_lamps.is_empty() {
            info!("Update for {} lamps", changed_lamps.len());
            for lamp in changed_lamps {
                //info!("lamp {}: {}", lamp.lampNo, lamp.state);
            }
        }

        let changed_solenoids = pinmame::get_changed_solenoids();
        if !changed_solenoids.is_empty() {
            info!("Update for {} solenoids", changed_solenoids.len());
            for solenoid in changed_solenoids {
                //info!("solenoid {}: {}", solenoid.solNo, solenoid.state);
            }
        }

        if let Some(display_layout) = tester.display_layout {
            match dmd_rx.try_recv() {
                Ok(display_data) => {
                    canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
                    canvas.clear();
                    dmd::render_dmd(&display_data, &display_layout, &mut canvas)?;
                    canvas.present();
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    error!("display_data channel disconnected");
                    break 'main;
                }
            }
        } else {
            info!("display_layout is None");
        }
    }

    pinmame::stop();

    Ok(())
}

fn describe_game(game: Game) -> String {
    format!(
        "name={}, description={}, manufacturer={}, year={}, flags={}, found={}",
        game.name, game.description, game.manufacturer, game.year, game.flags, game.found
    )
}

fn map_keycode(keycode: Keycode) -> Option<libpinmame::PINMAME_KEYCODE> {
    match keycode {
        Keycode::Num0 => Some(libpinmame::PINMAME_KEYCODE_NUMBER_0),
        Keycode::Num1 => Some(libpinmame::PINMAME_KEYCODE_NUMBER_1),
        Keycode::Num2 => Some(libpinmame::PINMAME_KEYCODE_NUMBER_2),
        Keycode::Num3 => Some(libpinmame::PINMAME_KEYCODE_NUMBER_3),
        Keycode::Num4 => Some(libpinmame::PINMAME_KEYCODE_NUMBER_4),
        Keycode::Num5 => Some(libpinmame::PINMAME_KEYCODE_NUMBER_5),
        Keycode::Num6 => Some(libpinmame::PINMAME_KEYCODE_NUMBER_6),
        Keycode::Num7 => Some(libpinmame::PINMAME_KEYCODE_NUMBER_7),
        Keycode::Num8 => Some(libpinmame::PINMAME_KEYCODE_NUMBER_8),
        Keycode::Num9 => Some(libpinmame::PINMAME_KEYCODE_NUMBER_9),
        Keycode::A => Some(libpinmame::PINMAME_KEYCODE_A),
        Keycode::B => Some(libpinmame::PINMAME_KEYCODE_B),
        Keycode::C => Some(libpinmame::PINMAME_KEYCODE_C),
        Keycode::D => Some(libpinmame::PINMAME_KEYCODE_D),
        Keycode::E => Some(libpinmame::PINMAME_KEYCODE_E),
        Keycode::F => Some(libpinmame::PINMAME_KEYCODE_F),
        Keycode::G => Some(libpinmame::PINMAME_KEYCODE_G),
        Keycode::H => Some(libpinmame::PINMAME_KEYCODE_H),
        Keycode::I => Some(libpinmame::PINMAME_KEYCODE_I),
        Keycode::J => Some(libpinmame::PINMAME_KEYCODE_J),
        Keycode::K => Some(libpinmame::PINMAME_KEYCODE_K),
        Keycode::L => Some(libpinmame::PINMAME_KEYCODE_L),
        Keycode::M => Some(libpinmame::PINMAME_KEYCODE_M),
        Keycode::N => Some(libpinmame::PINMAME_KEYCODE_N),
        Keycode::O => Some(libpinmame::PINMAME_KEYCODE_O),
        Keycode::P => Some(libpinmame::PINMAME_KEYCODE_P),
        Keycode::Q => Some(libpinmame::PINMAME_KEYCODE_Q),
        Keycode::R => Some(libpinmame::PINMAME_KEYCODE_R),
        Keycode::S => Some(libpinmame::PINMAME_KEYCODE_S),
        Keycode::T => Some(libpinmame::PINMAME_KEYCODE_T),
        Keycode::U => Some(libpinmame::PINMAME_KEYCODE_U),
        Keycode::V => Some(libpinmame::PINMAME_KEYCODE_V),
        Keycode::W => Some(libpinmame::PINMAME_KEYCODE_W),
        Keycode::X => Some(libpinmame::PINMAME_KEYCODE_X),
        Keycode::Y => Some(libpinmame::PINMAME_KEYCODE_Y),
        Keycode::Z => Some(libpinmame::PINMAME_KEYCODE_Z),
        Keycode::F1 => Some(libpinmame::PINMAME_KEYCODE_F1),
        Keycode::F2 => Some(libpinmame::PINMAME_KEYCODE_F2),
        Keycode::F3 => Some(libpinmame::PINMAME_KEYCODE_F3),
        Keycode::F4 => Some(libpinmame::PINMAME_KEYCODE_F4),
        Keycode::F5 => Some(libpinmame::PINMAME_KEYCODE_F5),
        Keycode::F6 => Some(libpinmame::PINMAME_KEYCODE_F6),
        Keycode::F7 => Some(libpinmame::PINMAME_KEYCODE_F7),
        Keycode::F8 => Some(libpinmame::PINMAME_KEYCODE_F8),
        Keycode::F9 => Some(libpinmame::PINMAME_KEYCODE_F9),
        Keycode::F10 => Some(libpinmame::PINMAME_KEYCODE_F10),
        Keycode::F11 => Some(libpinmame::PINMAME_KEYCODE_F11),
        Keycode::F12 => Some(libpinmame::PINMAME_KEYCODE_F12),
        Keycode::Kp0 => Some(libpinmame::PINMAME_KEYCODE_KEYPAD_0),
        Keycode::Kp1 => Some(libpinmame::PINMAME_KEYCODE_KEYPAD_1),
        Keycode::Kp2 => Some(libpinmame::PINMAME_KEYCODE_KEYPAD_2),
        Keycode::Kp3 => Some(libpinmame::PINMAME_KEYCODE_KEYPAD_3),
        Keycode::Kp4 => Some(libpinmame::PINMAME_KEYCODE_KEYPAD_4),
        Keycode::Kp5 => Some(libpinmame::PINMAME_KEYCODE_KEYPAD_5),
        Keycode::Kp6 => Some(libpinmame::PINMAME_KEYCODE_KEYPAD_6),
        Keycode::Kp7 => Some(libpinmame::PINMAME_KEYCODE_KEYPAD_7),
        Keycode::Kp8 => Some(libpinmame::PINMAME_KEYCODE_KEYPAD_8),
        Keycode::Kp9 => Some(libpinmame::PINMAME_KEYCODE_KEYPAD_9),
        Keycode::KpEnter => Some(libpinmame::PINMAME_KEYCODE_KEYPAD_ENTER),
        Keycode::KpPlus => Some(libpinmame::PINMAME_KEYCODE_KEYPAD_ADD),
        Keycode::KpMinus => Some(libpinmame::PINMAME_KEYCODE_KEYPAD_SUBTRACT),
        Keycode::KpMultiply => Some(libpinmame::PINMAME_KEYCODE_KEYPAD_MULTIPLY),
        Keycode::KpDivide => Some(libpinmame::PINMAME_KEYCODE_KEYPAD_DIVIDE),
        Keycode::LeftParen => Some(libpinmame::PINMAME_KEYCODE_LEFT_SHIFT),
        Keycode::RightParen => Some(libpinmame::PINMAME_KEYCODE_RIGHT_SHIFT),

        Keycode::Menu => Some(PINMAME_KEYCODE_MENU),
        Keycode::Escape => Some(PINMAME_KEYCODE_ESCAPE),
        _ => None,
    }
}

fn setup_sdl2(
    width: u32,
    height: u32,
) -> Result<(sdl2::Sdl, sdl2::render::Canvas<sdl2::video::Window>), String> {
    let sdl_context = sdl2::init()?;

    let video_subsys = sdl_context.video()?;
    let window = video_subsys
        .window("Pinmame rom tester", width, height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    // the canvas allows us to both manipulate the property of the window and to change its content
    // via hardware or software rendering. See CanvasBuilder for more info.
    let mut canvas = window
        .into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    Ok((sdl_context, canvas))
}
