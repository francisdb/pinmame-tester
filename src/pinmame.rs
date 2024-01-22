use std::ffi::{c_void, CStr, CString};

use log::{debug, error, info, trace, warn};

use crate::libpinmame::{
    PinmameConfig, PinmameGame, PinmameGetChangedLamps, PinmameGetChangedSolenoids, PinmameGetGame,
    PinmameGetGames, PinmameGetMaxLamps, PinmameGetMaxSolenoids, PinmameGetSwitch, PinmameIsPaused,
    PinmameIsRunning, PinmameLampState, PinmamePause, PinmameReset, PinmameRun, PinmameSetConfig,
    PinmameSetDmdMode, PinmameSetHandleKeyboard, PinmameSetHandleMechanics, PinmameSetSwitch,
    PinmameSetSwitches, PinmameSetUserData, PinmameSolenoidState, PinmameStop, PinmameSwitchState,
    PINMAME_DMD_MODE, PINMAME_DMD_MODE_PINMAME_DMD_MODE_BRIGHTNESS,
    PINMAME_DMD_MODE_PINMAME_DMD_MODE_RAW, PINMAME_LOG_LEVEL,
    PINMAME_LOG_LEVEL_PINMAME_LOG_LEVEL_DEBUG, PINMAME_LOG_LEVEL_PINMAME_LOG_LEVEL_ERROR,
    PINMAME_LOG_LEVEL_PINMAME_LOG_LEVEL_INFO, PINMAME_STATUS,
    PINMAME_STATUS_PINMAME_STATUS_CONFIG_NOT_SET,
    PINMAME_STATUS_PINMAME_STATUS_EMULATOR_NOT_RUNNING,
    PINMAME_STATUS_PINMAME_STATUS_GAME_ALREADY_RUNNING,
    PINMAME_STATUS_PINMAME_STATUS_GAME_NOT_FOUND,
    PINMAME_STATUS_PINMAME_STATUS_MECH_HANDLE_MECHANICS,
    PINMAME_STATUS_PINMAME_STATUS_MECH_NO_INVALID, PINMAME_STATUS_PINMAME_STATUS_OK,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PinmameStatus {
    Ok,
    ConfigNotSet,
    GameNotFound,
    GameAlreadyRunning,
    EmulatorNotRunning,
    MechHandleMechanics,
    MechNoInvalid,
}

impl From<PINMAME_STATUS> for PinmameStatus {
    fn from(status: u32) -> Self {
        match status {
            PINMAME_STATUS_PINMAME_STATUS_OK => PinmameStatus::Ok,
            PINMAME_STATUS_PINMAME_STATUS_CONFIG_NOT_SET => PinmameStatus::ConfigNotSet,
            PINMAME_STATUS_PINMAME_STATUS_GAME_NOT_FOUND => PinmameStatus::GameNotFound,
            PINMAME_STATUS_PINMAME_STATUS_GAME_ALREADY_RUNNING => PinmameStatus::GameAlreadyRunning,
            PINMAME_STATUS_PINMAME_STATUS_EMULATOR_NOT_RUNNING => PinmameStatus::EmulatorNotRunning,
            PINMAME_STATUS_PINMAME_STATUS_MECH_HANDLE_MECHANICS => {
                PinmameStatus::MechHandleMechanics
            }
            PINMAME_STATUS_PINMAME_STATUS_MECH_NO_INVALID => PinmameStatus::MechNoInvalid,
            _ => unreachable!("Unknown status code"),
        }
    }
}

pub struct Game {
    pub name: String,
    pub clone_of: String,
    pub description: String,
    pub year: String,
    pub manufacturer: String,
    pub flags: u32,
    pub found: i32,
}
impl From<PinmameGame> for Game {
    fn from(game: PinmameGame) -> Self {
        Game {
            name: unsafe { CStr::from_ptr(game.name).to_str().unwrap().to_string() },
            clone_of: unsafe { CStr::from_ptr(game.clone_of).to_str().unwrap().to_string() },
            description: unsafe {
                CStr::from_ptr(game.description)
                    .to_str()
                    .unwrap()
                    .to_string()
            },
            year: unsafe { CStr::from_ptr(game.year).to_str().unwrap().to_string() },
            manufacturer: unsafe {
                CStr::from_ptr(game.manufacturer)
                    .to_str()
                    .unwrap()
                    .to_string()
            },
            flags: game.flags,
            found: game.found,
        }
    }
}

pub enum DmdMode {
    Brightness,
    Raw,
}

impl From<DmdMode> for PINMAME_DMD_MODE {
    fn from(dmd_mode: DmdMode) -> Self {
        match dmd_mode {
            DmdMode::Brightness => PINMAME_DMD_MODE_PINMAME_DMD_MODE_BRIGHTNESS,
            DmdMode::Raw => PINMAME_DMD_MODE_PINMAME_DMD_MODE_RAW,
        }
    }
}

pub fn set_config(config: &PinmameConfig) {
    unsafe { PinmameSetConfig(config) }
}

pub fn set_user_data(user_data: *const std::ffi::c_void) {
    unsafe { PinmameSetUserData(user_data) }
}

pub fn set_handle_keyboard(handle: bool) {
    unsafe { PinmameSetHandleKeyboard(handle as i32) }
}

pub fn set_handle_mechanics(handle: bool) {
    unsafe { PinmameSetHandleMechanics(handle as i32) }
}

pub fn set_dmd_mode(dmd_mode: DmdMode) {
    unsafe { PinmameSetDmdMode(dmd_mode.into()) }
}

pub fn is_running() -> bool {
    unsafe { PinmameIsRunning() > 0 }
}

pub fn stop() {
    unsafe { PinmameStop() };
}

pub fn reset() {
    unsafe { PinmameReset() };
}

pub fn pause() -> Result<(), PINMAME_STATUS> {
    let status = unsafe { PinmamePause(1) };
    match status {
        PINMAME_STATUS_PINMAME_STATUS_OK => Ok(()),
        _ => Err(status.into()),
    }
}
pub fn continue_() -> Result<(), PINMAME_STATUS> {
    let status = unsafe { PinmamePause(0) };
    match status {
        PINMAME_STATUS_PINMAME_STATUS_OK => Ok(()),
        _ => Err(status.into()),
    }
}

pub fn is_paused() -> bool {
    unsafe { PinmameIsPaused() > 0 }
}

pub fn get_games() -> Result<Vec<Game>, PinmameStatus> {
    let games_user_data = GamesUserData { games: vec![] };
    let status = unsafe {
        PinmameGetGames(
            Some(games_callback),
            &games_user_data as *const _ as *mut c_void,
        )
    }
    .into();
    if status != PinmameStatus::Ok {
        return Err(status);
    }
    Ok(games_user_data.games)
}

pub fn get_game(p_name: &str) -> Result<Game, PinmameStatus> {
    let p_name = CString::new(p_name).unwrap();
    let game_user_data = GameUserData { game: None };
    let status: PinmameStatus = unsafe {
        PinmameGetGame(
            p_name.as_ptr(),
            Some(game_callback),
            &game_user_data as *const _ as *mut c_void,
        )
    }
    .into();
    if status != PinmameStatus::Ok {
        return Err(status);
    }
    Ok(game_user_data.game.unwrap())
}

pub fn run(p_name: &str) -> PinmameStatus {
    let p_name = CString::new(p_name).unwrap();
    unsafe { PinmameRun(p_name.as_ptr()) }.into()
}

pub fn get_max_lamps() -> i32 {
    unsafe { PinmameGetMaxLamps() }
}

pub fn get_max_solenoids() -> i32 {
    unsafe { PinmameGetMaxSolenoids() }
}

pub fn set_switch(switch_no: i32, state: i32) {
    unsafe { PinmameSetSwitch(switch_no, state) }
}

pub fn get_switch(switch_no: i32) -> i32 {
    unsafe { PinmameGetSwitch(switch_no) }
}

pub fn set_switches(switches: &[PinmameSwitchState]) {
    unsafe { PinmameSetSwitches(switches.as_ptr(), switches.len() as i32) };
}

pub fn get_changed_lamps() -> Vec<PinmameLampState> {
    let max_lamps = get_max_lamps();
    // TODO could be more efficient to keep this vector around between invocations
    let lamps_changed: Vec<PinmameLampState> = vec![
        PinmameLampState {
            lampNo: 0,
            state: 0
        };
        max_lamps as usize
    ];
    let lamps_state: *mut PinmameLampState = lamps_changed.as_ptr() as *mut PinmameLampState;
    let num = unsafe { PinmameGetChangedLamps(lamps_state) };

    if num == -1 {
        return vec![];
    }
    let states = unsafe { std::slice::from_raw_parts(lamps_state, num.try_into().unwrap()) };
    states.to_vec()
}

pub fn get_changed_solenoids() -> Vec<PinmameSolenoidState> {
    let max_solenoids = get_max_solenoids();
    // TODO could be more efficient to keep this vector around between invocations
    let solenoids_changed: Vec<PinmameSolenoidState> =
        vec![PinmameSolenoidState { solNo: 0, state: 0 }; max_solenoids as usize];
    let solenoids_state: *mut PinmameSolenoidState =
        solenoids_changed.as_ptr() as *mut PinmameSolenoidState;
    let num = unsafe { PinmameGetChangedSolenoids(solenoids_state) };

    if num == -1 {
        return vec![];
    }
    let states = unsafe { std::slice::from_raw_parts(solenoids_state, num.try_into().unwrap()) };
    states.to_vec()
}

struct GameUserData {
    game: Option<Game>,
}

extern "C" fn game_callback(game: *mut PinmameGame, mut _p_user_data: *const c_void) {
    unsafe {
        let game = &mut *game;
        let p_user_data = &mut *(_p_user_data as *mut GameUserData);
        p_user_data.game = Some((*game).into());
    }
}

struct GamesUserData {
    games: Vec<Game>,
}

extern "C" fn games_callback(game: *mut PinmameGame, mut _p_user_data: *const c_void) {
    unsafe {
        let game = &mut *game;
        let p_user_data = &mut *(_p_user_data as *mut GamesUserData);
        p_user_data.games.push((*game).into());
    }
}

//TODO make private
pub extern "C" fn pinmame_on_solenoid_updated_callback(
    solenoid_state: *mut PinmameSolenoidState,
    _user_data: *const ::std::os::raw::c_void,
) {
    unsafe {
        trace!(
            "OnSolenoidUpdated: solenoid={}, state={}",
            (*solenoid_state).solNo,
            (*solenoid_state).state
        );
    }
}

//TODO make private
pub unsafe extern "C" fn pinmame_on_console_data_updated_callback(
    _data: *mut ::std::os::raw::c_void,
    size: i32,
    _user_data: *const ::std::os::raw::c_void,
) {
    info!("OnConsoleDataUpdated: size={}", size);
}

// see https://github.com/rust-lang/rust-bindgen/issues/2631
#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
type VaListType = *mut crate::libpinmame::__va_list_tag;
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
type VaListType = crate::libpinmame::va_list;

//TODO make private
pub unsafe extern "C" fn pinmame_on_log_message_callback(
    log_level: u32,
    format: *const ::std::os::raw::c_char,
    args: VaListType,
    _user_data: *const ::std::os::raw::c_void,
) {
    let str = unsafe { vsprintf::vsprintf(format, args).unwrap() };
    on_log_message(log_level, str);
}

fn on_log_message(log_level: PINMAME_LOG_LEVEL, str: String) {
    // if message contains ERROR, log it as error
    if str.contains("ERROR") {
        error!(target: "pinmame", "{}", str);
        return;
    }
    match log_level {
        PINMAME_LOG_LEVEL_PINMAME_LOG_LEVEL_DEBUG => {
            debug!(target: "pinmame", "{}", str);
        }
        PINMAME_LOG_LEVEL_PINMAME_LOG_LEVEL_INFO => {
            info!(target: "pinmame", "{}", str);
        }
        PINMAME_LOG_LEVEL_PINMAME_LOG_LEVEL_ERROR => {
            error!(target: "pinmame", "{}", str);
        }
        _ => warn!("Unknown log level: {}", log_level),
    }
}
