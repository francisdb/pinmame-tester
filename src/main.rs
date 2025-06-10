#[macro_use]
extern crate lazy_static;

use log::{debug, error, info, trace, warn};
use std::{
    collections::HashMap,
    ffi::{c_char, c_void, CString},
    sync::mpsc,
};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::{
    audio::{AudioQueue, AudioSpecDesired},
    mouse::MouseButton,
    pixels,
};

use libpinmame::{
    PinmameAudioInfo, PinmameConfig, PinmameDisplayLayout, PinmameMechInfo,
    PINMAME_AUDIO_FORMAT_PINMAME_AUDIO_FORMAT_INT16, PINMAME_KEYCODE_PINMAME_KEYCODE_ESCAPE,
    PINMAME_KEYCODE_PINMAME_KEYCODE_MENU, PINMAME_KEYCODE_PINMAME_KEYCODE_Q,
};
use pinmame::{Game, PinmameStatus};

use crate::{
    db::SwitchIndex,
    keyboard::map_keycode,
    pinmame::{
        pinmame_on_console_data_updated_callback, pinmame_on_log_message_callback,
        pinmame_on_solenoid_updated_callback, DmdMode,
    },
};
mod db;
mod dmd;
mod keyboard;
#[allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals
)]
mod libpinmame;
pub mod pinmame;
mod switches;

extern "C" fn pinmame_on_state_updated_callback(state: i32, _p_user_data: *const c_void) {
    info!("OnStateUpdated(): state={}", state);

    if state == 0 {
        warn!("OnStateUpdated(): state=0, exiting");
        std::process::exit(1);
    } else {
        // We had to come up with our own defaults
        // Are these correct?
        let mech_config = libpinmame::PinmameMechConfig {
            type_: (libpinmame::PINMAME_MECH_FLAGS_PINMAME_MECH_FLAGS_NONLINEAR
                | libpinmame::PINMAME_MECH_FLAGS_PINMAME_MECH_FLAGS_REVERSE
                | libpinmame::PINMAME_MECH_FLAGS_PINMAME_MECH_FLAGS_ONESOL)
                as i32,
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
    let layout = unsafe { *display_layout };

    info!(
        "OnDisplayAvailable(): index={}, displayCount={}, type={}, top={}, left={}, width={}, height={}, depth={}, length={}",
        index,
        display_count,
        layout.type_,
        layout.top,
        layout.left,
        layout.width,
        layout.height,
        layout.depth,
        layout.length
    );
    // set the display layout
    let tester: &mut Tester = unsafe { &mut *(_p_user_data as *mut Tester) };
    tester.display_layout = Some(layout);
}

unsafe extern "C" fn pinmame_on_display_updated_callback(
    index: i32,
    _display_data: *mut ::std::os::raw::c_void,
    display_layout: *mut PinmameDisplayLayout,
    _user_data: *const ::std::os::raw::c_void,
) {
    let display_layout_ref = unsafe { display_layout.as_ref().unwrap() };
    trace!(
        "OnDisplayUpdated(): index={}, type={}, top={}, left={}, width={}, height={}, depth={}, length={}",
        index,
        display_layout_ref.type_,
        display_layout_ref.top,
        display_layout_ref.left,
        display_layout_ref.width,
        display_layout_ref.height,
        display_layout_ref.depth,
        display_layout_ref.length
    );

    if !_display_data.is_null() {
        if (display_layout_ref.type_ & libpinmame::PINMAME_DISPLAY_TYPE_PINMAME_DISPLAY_TYPE_DMD)
            == libpinmame::PINMAME_DISPLAY_TYPE_PINMAME_DISPLAY_TYPE_DMD
        {
            let tester: &mut Tester = unsafe { &mut *(_user_data as *mut Tester) };
            match tester.display_data.send(unsafe {
                std::slice::from_raw_parts_mut(
                    _display_data as *mut u8,
                    display_layout_ref.width as usize * display_layout_ref.height as usize,
                )
                .to_owned()
            }) {
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
    audio_info: *mut PinmameAudioInfo,
    _user_data: *const ::std::os::raw::c_void,
) -> i32 {
    let audio_info = unsafe { audio_info.as_ref().unwrap() };
    let format = match audio_info.format {
        libpinmame::PINMAME_AUDIO_FORMAT_PINMAME_AUDIO_FORMAT_INT16 => "int16",
        libpinmame::PINMAME_AUDIO_FORMAT_PINMAME_AUDIO_FORMAT_FLOAT => "float",
        other => unreachable!("Unknown audio format: {}", other),
    };
    info!(
        "OnAudioAvailable(): format={}, channels={}, sampleRate={}, framesPerSecond={}, samplesPerFrame={}, bufferSize={}",
        format,
        audio_info.channels,
        audio_info.sampleRate,
        audio_info.framesPerSecond,
        audio_info.samplesPerFrame,
        audio_info.bufferSize
    );

    let tester: &mut Tester = unsafe { &mut *(_user_data as *mut Tester) };
    tester.audio_info = Some(*audio_info);

    audio_info.samplesPerFrame
}

//TODO make private
unsafe extern "C" fn pinmame_on_mech_available_callback(
    mech_no: ::std::os::raw::c_int,
    mech_info: *mut libpinmame::PinmameMechInfo,
    _user_data: *const ::std::os::raw::c_void,
) {
    // TODO not sure we need to clone here
    // TODO do we need to free this memory?
    let mech_info = unsafe { mech_info.as_ref().expect("mech_info is null") };
    let safe_mech_info = PinmameMechInfo {
        type_: mech_info.type_,
        length: mech_info.length,
        steps: mech_info.steps,
        pos: mech_info.pos,
        speed: mech_info.speed,
    };

    info!(
        "OnMechAvailable(): mechNo={}, type={}, length={}, steps={}, pos={}, speed={}",
        mech_no,
        safe_mech_info.type_,
        safe_mech_info.length,
        safe_mech_info.steps,
        safe_mech_info.pos,
        safe_mech_info.speed
    );
    let tester: &mut Tester = unsafe { &mut *(_user_data as *mut Tester) };
    tester.mech_info.push(safe_mech_info);
}

//TODO make private
pub unsafe extern "C" fn pinmame_on_mech_updated_callback(
    mech_no: i32,
    mech_info: *mut PinmameMechInfo,
    _user_data: *const ::std::os::raw::c_void,
) {
    let mech_info_ref = unsafe { mech_info.as_ref().unwrap() };
    trace!(
        "OnMechUpdated: mechNo={}, type={}, length={}, steps={}, pos={}, speed={}",
        mech_no,
        mech_info_ref.type_,
        mech_info_ref.length,
        mech_info_ref.steps,
        mech_info_ref.pos,
        mech_info_ref.speed
    );
    let tester: &mut Tester = unsafe { &mut *(_user_data as *mut Tester) };
    tester.mech_info[mech_no as usize] = unsafe { *mech_info };
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

extern "C" fn pinmame_on_sound_command_callback(
    board_no: ::std::os::raw::c_int,
    cmd: ::std::os::raw::c_int,
    _p_user_data: *const ::std::os::raw::c_void,
) {
    // TODO
    info!("OnSoundCommand(): boardNo={}, cmd={}", board_no, cmd);
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
    keyboard_state: [bool; (PINMAME_KEYCODE_PINMAME_KEYCODE_MENU + 1) as usize],
    mech_info: Vec<PinmameMechInfo>,
    lamps: Vec<bool>,
    solenoids: Vec<bool>,
    switches: HashMap<u32, bool>,
}

const SCREEN_WIDTH: u32 = 800; // PIXELS_WIDTH * (PIXEL_SIZE + 1);
const SCREEN_HEIGHT: u32 = 600; // PIXELS_HEIGHT * (PIXEL_SIZE + 1);

fn main() -> Result<(), String> {
    // run me like this: RUST_LOG=info cargo run

    // info in T2 here
    // https://github.com/VisualPinball/VisualPinball.Engine.PinMAME/blob/master/VisualPinball.Engine.PinMAME/Games/Terminator2.cs

    pretty_env_logger::init();

    // TODO we should get this from the loaded rom, or update the window when we get it
    let (sdl_context, mut canvas) = setup_sdl2(SCREEN_WIDTH, SCREEN_HEIGHT)?;
    let mut events = sdl_context.event_pump()?;
    let audio_subsystem = sdl_context.audio()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    // Load a font
    let font = ttf_context.load_font(
        "res/roboto/Roboto-Regular.ttf",
        (12_f32 * canvas.scale().0) as u16,
    )?;
    //font.set_style(sdl2::ttf::FontStyle::BOLD);

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
        keyboard_state: [false; (PINMAME_KEYCODE_PINMAME_KEYCODE_MENU + 1) as usize],
        mech_info: Vec::new(),
        lamps: Vec::new(),
        solenoids: Vec::new(),
        switches: HashMap::new(),
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
        audioFormat: PINMAME_AUDIO_FORMAT_PINMAME_AUDIO_FORMAT_INT16,
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
        cb_OnSoundCommand: Some(pinmame_on_sound_command_callback),
    };

    //PinmameRun("mm_109c"); // Medieval Madness
    //PinmameRun("fh_906h"); // FunHouse
    //PinmameRun("hh7"); // Haunted House, 7 displays
    //PinmameRun("rescu911"); // Rescue 911 (Switch short return 5 error)
    //PinmameRun("tf_180h"); // Transformers
    //PinmameRun("flashgdn");
    //PinmameRun("fourx4"); // 4x4, 2 displays?
    //PinmameRun("ripleys");
    //PinmameRun("fh_l9"); // FunHouse
    //PinmameRun("acd_170hc"); // ACDC
    //PinmameRun("snspares");
    // xfiles - sound is messed up, indicates 2 channels
    // hook_501 - sound is messed up, indicates 2 channels
    // barbwire - sound is messed up, indicates 2 channels
    // cv_20h - cirqus voltaire
    // totan_14 - Tales of the Arabian Nights

    //let terminator_2_game_name = "t2_l8";
    //let medieval_madness_game_name = "mm_109c";
    //let fourx4_game_name = "fourx4";

    // Terminator 2
    let p_name = "t2_l8";
    let switch_index = &db::T2_SWITCHES;

    // Medieval Madness
    // let p_name = "mm_109c";
    // let switch_index = &db::MM_SWITCHES;

    pinmame::set_config(&config);

    pinmame::set_user_data(&tester as *const Tester as *const std::ffi::c_void);
    pinmame::set_handle_keyboard(false);
    pinmame::set_handle_mechanics(true);

    pinmame::set_dmd_mode(DmdMode::Raw);
    match pinmame::get_games() {
        Ok(games) => {
            info!("Found {} games", games.len());
            let mut games = games;
            games.sort_by(|a, b| a.name.cmp(&b.name));
            // for game in games {
            //     info!("  {}", describe_game(game));
            // }
        }
        Err(status) => {
            error!("get_games() failed: {:?}", status);
        }
    }

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
    tester.lamps = vec![false; max_lamps as usize];

    let max_solenoids = pinmame::get_max_solenoids();
    info!("max_solenoids: {}", max_solenoids);
    tester.solenoids = vec![false; max_solenoids as usize];

    // Close the coin door for Medieval Madness
    //pinmame::set_switch(22, 1);

    // Rescu911 Switch short return 5
    // set all switches to 1
    // for i in 0..63 {
    //     pinmame::set_switch(i, 1);
    // }

    let mut display_data = vec![0; 128 * 32];

    'main: loop {
        // get the inputs here
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,

                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    match map_keycode(keycode) {
                        Some(keycode) => {
                            tester.keyboard_state[keycode as usize] = true;
                        }
                        None => warn!("KeyDown keycode not mapped: {:?}", keycode),
                    }
                    match keycode {
                        Keycode::Escape => {
                            tester.keyboard_state
                                [PINMAME_KEYCODE_PINMAME_KEYCODE_ESCAPE as usize] = true;
                            break 'main;
                        }
                        Keycode::Q => {
                            tester.keyboard_state[PINMAME_KEYCODE_PINMAME_KEYCODE_Q as usize] =
                                true;
                            break 'main;
                        }
                        _ => {}
                    }
                    //println!("key down: {:?}", keycode);
                }
                Event::KeyUp { keycode, .. } => match keycode {
                    Some(keycode) => match map_keycode(keycode) {
                        Some(keycode) => {
                            tester.keyboard_state[keycode as usize] = false;
                        }
                        None => warn!("KeyUp keycode not mapped: {:?}", keycode),
                    },
                    None => (),
                },

                Event::MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {
                    // canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
                    // canvas.clear();
                    // let color = pixels::Color::RGB(x as u8, y as u8, 255);
                    // let _ = canvas.line(lastx, lasty, x as i16, y as i16, color);
                    // lastx = x as i16;
                    // lasty = y as i16;
                    // println!("mouse btn down at ({},{})", x, y);
                    let s = pinmame::get_switch(16);
                    println!("switch 16: {}", s);

                    if let Some(display_layout) = tester.display_layout {
                        let switches_at_x = 0;
                        let switches_at_y =
                            (dmd::dmd_height(&display_layout) + 10 + 100 + 50) as i32;
                        if let Some(switch) = switches::switch_id_for_mouse(
                            x - switches_at_x,
                            y - switches_at_y,
                            switch_index,
                        ) {
                            match mouse_btn {
                                MouseButton::Left => {
                                    println!("switch {} true", switch);
                                    tester.switches.insert(switch, true);
                                    pinmame::set_switch(switch as i32, 1);
                                }
                                MouseButton::Right => {
                                    let switched =
                                        *tester.switches.get(&switch).unwrap_or(&false) ^ true;
                                    println!("switch {} toggled to {}", switch, switched);
                                    tester.switches.insert(switch, switched);

                                    pinmame::set_switch(switch as i32, switched as i32);
                                }
                                _ => (),
                            }
                        }
                    }
                }

                Event::MouseButtonUp {
                    x, y, mouse_btn, ..
                } => {
                    // println!("mouse btn up at ({},{})", x, y);

                    if let Some(display_layout) = tester.display_layout {
                        let switches_at_x = 0;
                        let switches_at_y =
                            (dmd::dmd_height(&display_layout) + 10 + 100 + 50) as i32;
                        if let Some(switch) = switches::switch_id_for_mouse(
                            x - switches_at_x,
                            y - switches_at_y,
                            switch_index,
                        ) {
                            match mouse_btn {
                                MouseButton::Left => {
                                    tester.switches.insert(switch, false);
                                    println!("switch {} false", switch);
                                    pinmame::set_switch(switch as i32, 0);
                                }
                                _ => (),
                            }
                        }
                    }
                }

                _ => {}
            }
        }

        // update the game loop here

        let changed_lamps = pinmame::get_changed_lamps();
        if !changed_lamps.is_empty() {
            //info!("Update for {} lamps", changed_lamps.len());
            for lamp in changed_lamps {
                if lamp.state != 0 && lamp.state != 255 {
                    info!("lamp {}: {}", lamp.lampNo, lamp.state);
                }
                tester.lamps[lamp.lampNo as usize] = lamp.state != 0;
                //info!("lamp {}: {}", lamp.lampNo, lamp.state);
            }
        }

        // TODO we also have a callback for solenoid updates, which should we be using?
        let changed_solenoids = pinmame::get_changed_solenoids();
        if !changed_solenoids.is_empty() {
            //info!("Update for {} solenoids", changed_solenoids.len());
            for solenoid in changed_solenoids {
                if solenoid.state != 0 && solenoid.state != 1 {
                    info!("solenoid {}: {}", solenoid.solNo, solenoid.state);
                }
                tester.solenoids[solenoid.solNo as usize] = solenoid.state != 0;
                //info!("solenoid {}: {}", solenoid.solNo, solenoid.state);
            }
        }

        let lamp_size = 8;

        if let Some(display_layout) = tester.display_layout {
            match dmd_rx.try_recv() {
                Ok(received_display_data) => {
                    display_data = received_display_data;
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    error!("display_data channel disconnected");
                    break 'main;
                }
            }
            render(
                &mut canvas,
                &font,
                &display_data,
                display_layout,
                &tester,
                lamp_size,
                switch_index,
            )?;
        } else {
            info!("display_layout is None");
        }
    }

    pinmame::stop();

    Ok(())
}

fn render(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    font: &sdl2::ttf::Font,
    display_data: &[u8],
    display_layout: PinmameDisplayLayout,
    tester: &Tester,
    lamp_size: u32,
    switch_index: &SwitchIndex,
) -> Result<(), String> {
    canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
    canvas.clear();
    dmd::render_dmd(0, 0, display_data, &display_layout, canvas)?;
    dmd::render_lights(
        0,
        dmd::dmd_height(&display_layout) + 10,
        &tester.lamps,
        canvas,
        lamp_size,
    )?;
    dmd::render_solenoids(
        300,
        dmd::dmd_height(&display_layout) + 10,
        &tester.solenoids,
        canvas,
        lamp_size,
    )?;
    dmd::render_mechs(
        0,
        dmd::dmd_height(&display_layout) + 10 + 100,
        &tester.mech_info,
        canvas,
    )?;

    switches::render_switches(
        0,
        dmd::dmd_height(&display_layout) + 10 + 100 + 50,
        canvas,
        font,
        switch_index,
        &tester.switches,
    )?;

    canvas.present();
    Ok(())
}

fn describe_game(game: Game) -> String {
    format!(
        "name={}, description={}, manufacturer={}, year={}, flags={}, found={}",
        game.name, game.description, game.manufacturer, game.year, game.flags, game.found
    )
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
        .allow_highdpi()
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

    // SDL_GetRendererOutputSize(render, &rw, &rh);
    // in rust
    let (rw, rh) = canvas.output_size()?;
    let width_scale = (rw as f32) / (width as f32);
    let height_scale = (rh as f32) / (height as f32);

    debug!("width_scale: {}", width_scale);
    debug!("height_scale: {}", height_scale);
    if width_scale != height_scale {
        error!("WARNING: width scale != height scale");
    }

    // SDL_RenderSetScale(render, widthScale, heightScale);
    // in rust
    canvas.set_scale(width_scale, height_scale)?;

    canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    Ok((sdl_context, canvas))
}
