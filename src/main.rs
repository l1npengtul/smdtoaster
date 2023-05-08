#![feature(abi_avr_interrupt)]
#![no_std]

extern crate alloc;

use crate::lcd::LCDHardware;
use crate::temperature::Temperature;
use ::lcd::Display;
use avr_delay::delay_ms;
use avrd::atmega328::PORTB;
use avrd::current::{EEARH, EEARL};
use core::cmp::Ordering;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicU64, Ordering as MemOrdering};
use ruduino::cores::current::{EEAR, EECR, EEDR};

use crate::profile::{CurvePoint, Profile, Profiles};
use ruduino::cores::current::port::{C0, C1, C2, C3, C4, C6, D2, D3};
use ruduino::{Pin, Register};
use sb_rotary_encoder::{Direction, RotaryEncoder};

mod lcd;
mod profile;
mod temperature;
mod ui;

#[derive(Default)]
enum UiState {
    #[default]
    MainMenu,
    ProfileEdit,
    OvenRun,
}

#[derive(Default)]
enum ProfileEditSubMenus {
    #[default]
    ProfileSelect,
    ProfileElementSelect,
    ProfileNameEdit,
    ProfilePointSelect,
    ProfilePointSelectElementEdit,
    ProfilePointTempEdit,
    ProfilePointTimeEdit,
    ProfilePointDisabledEdit,
    ProfileExitConfirmMenu,
    ProfileWriteConfirmMenu,
}

#[derive(Default)]
enum OvenRunSubMenus {
    #[default]
    OvenProfileSelect,
    OvenProfileRunConfirm,
    OvenProfileRunningMenu,
    OvenProfileCancelRunningMenu,
}

const PULSE_DIVIDER: i32 = 4;

static OYASUMI_TIME: AtomicU64 = AtomicU64::new(0);

#[no_mangle]
pub unsafe extern "avr-interrupt" fn _ivr_timer1_compare_a() {
    OYASUMI_TIME.fetch_add(1, MemOrdering::SeqCst);
}

fn main() {
    // 1602 LCD
    let hw = LCDHardware {};
    let mut display = Display::new(hw);

    // K type
    Temperature::setup();

    type FanRelay = C3;
    type HeaterRelay = C4;
    type ButtonPin = C2;
    type SWPin = C1;
    type APin = D3;
    type BPin = D2;

    FanRelay::set_output();
    HeaterRelay::set_output();
    ButtonPin::set_input();
    SWPin::set_input();
    APin::set_input();
    BPin::set_input();

    const DESIRED_HZ_TIM1: f64 = 2.0;
    const TIM1_PRESCALER: u64 = 1024;
    const INTERRUPT_EVERY_1_HZ_1024_PRESCALER: u16 = ((ruduino::config::CPU_FREQUENCY_HZ as f64
        / (DESIRED_HZ_TIM1 * TIM1_PRESCALER as f64))
        as u64
        - 1) as u16;

    timer1::Timer::new()
        .waveform_generation_mode(timer1::WaveformGenerationMode::ClearOnTimerMatchOutputCompare)
        .clock_source(timer1::ClockSource::Prescale1024)
        .output_compare_1(Some(INTERRUPT_EVERY_1_HZ_1024_PRESCALER))
        .configure();

    // read first 2 bytes of EEPROM
    unsafe {
        *EEARH = 0b0000_0000_u8;
    }
    unsafe {
        *EEARL = 0b0000_0000_u8;
    }
    unsafe { EECR::set(EECR::EERE) }
    let low_byte = EEDR::read();
    unsafe {
        *EEARL = 0b0000_0001_u8;
    }
    unsafe { EECR::set(EECR::EERE) }
    let high_byte = EEDR::read();
    let num_bytes = u16::from_le_bytes([high_byte, low_byte]) + 3;

    let mut profiles = {
        let mut data = [0_u8; 1000];

        for addr in 3..num_bytes {
            let split: [u8; 2] = addr.to_le_bytes();
            unsafe {
                *EEARH = split[0];
            }
            unsafe {
                *EEARL = split[1];
            }
            unsafe { EECR::set(EECR::EERE) }
            data[addr] = EEDR::read();
        }

        postcard::from_bytes::<Profiles>(&data[0..num_bytes - 3]).unwrap()
    };

    let mut rotary = RotaryEncoder::new();

    let mut clocks = 0_u128;
    const DISPLAY_UPDATE: u8 = 20;

    let mut ui_counter = 0_u8;
    let mut ui_state = UiState::MainMenu;
    let mut profile_edit_state = ProfileEditSubMenus::default();
    let mut run_profile_idx = 0;
    let mut profile_edit_num = 7_u8;
    let mut profile_editing_temp_profile = Profile {
        name: [b' ', b' ', b' ', b' ', b' ', b' '],
        points: [
            CurvePoint {
                temp: 0,
                time_seconds: 0,
                disabled: true,
            },
            CurvePoint {
                temp: 0,
                time_seconds: 0,
                disabled: true,
            },
            CurvePoint {
                temp: 0,
                time_seconds: 0,
                disabled: true,
            },
            CurvePoint {
                temp: 0,
                time_seconds: 0,
                disabled: true,
            },
            CurvePoint {
                temp: 0,
                time_seconds: 0,
                disabled: true,
            },
            CurvePoint {
                temp: 0,
                time_seconds: 0,
                disabled: true,
            },
        ],
    };
    let mut idx = 0;
    let mut idx1 = 0;
    let mut oven_run_state = OvenRunSubMenus::default();
    let mut changed = false;
    let mut running_oven = false;
    let mut time_left = 0;
    let mut current_start_time = 0;
    let mut current_pt = 0;

    write!(display, "BOOTING...").unwrap();
    delay_ms(2000);

    let mut input_a = APin::is_high();
    let mut input_b = BPin::is_high();
    let mut button = ButtonPin::is_high();
    let mut alt_button = SWPin::is_high();
    let mut temp = Temperature::read_temperature();
    let mut direction = Direction::Clockwise;

    loop {
        // read inputs
        temp = Temperature::read_temperature();
        input_a = APin::is_high();
        input_b = BPin::is_high();
        button = ButtonPin::is_high();
        alt_button = SWPin::is_high();

        if let Some(event) = rotary.update(input_a, input_b, None, PULSE_DIVIDER) {
            direction = event.direction();
            match direction {
                Direction::Clockwise => {
                    if ui_counter != 255 {
                        ui_counter += 1;
                    }
                }
                Direction::CounterClockwise => {
                    if ui_counter != 0 {
                        ui_counter -= 1;
                    }
                }
            }
        }

        if running_oven {
            if !FanRelay::is_high() {
                FanRelay::set_high();
            }
            // Temperature, decide if our current point
            if let Some(profile) = &profiles.profiles[run_profile_idx as usize] {
                let next_point = profile.points[current_pt + 1];
                let this_point = profile.points[current_pt];
                let time = OYASUMI_TIME.load(MemOrdering::SeqCst);
                time_left = time - current_start_time;

                let target = (next_point.temp - this_point.temp)
                    / (next_point.time_seconds - this_point.time_seconds);
                match target.cmp(&temp) {
                    Ordering::Less => {
                        HeaterRelay::set_low();
                    }
                    Ordering::Equal => {
                        HeaterRelay::set_high();
                    }
                    Ordering::Greater => {
                        HeaterRelay::set_high();
                    }
                }

                if next_point.time_seconds <= time as u16 {
                    current_pt += 1;
                    if current_pt >= 5 {
                        oven_run_state = OvenRunSubMenus::OvenProfileSelect;
                        ui_state = UiState::MainMenu;
                        running_oven = false;
                        time_left = 0;
                        current_start_time = 0;
                    }
                }
            }
        }

        if clocks % display_update == 0 {
            match ui_state {
                UiState::MainMenu => {
                    ui::main_menu(&mut display, temp, ui_counter, changed);
                    if changed {
                        changed = false;
                    }
                    if button {
                        match ui_counter {
                            0 => {
                                ui_state = UiState::OvenRun;
                                changed = true
                            }
                            1 => {
                                ui_state = UiState::ProfileEdit;
                                changed = true
                            }
                            _ => {
                                ui_counter = 0;
                            }
                        }
                        ui_counter = 0;
                    }
                }
                UiState::ProfileEdit => match profile_edit_state {
                    ProfileEditSubMenus::ProfileSelect => {
                        let rst = ui::select_edit_profile_menu(
                            &mut display,
                            ui_counter,
                            &profiles,
                            changed,
                        );
                        if rst {
                            ui_counter = 0;
                        }
                        if changed {
                            changed = false;
                        }
                        if button {
                            if ui_counter >= 16 {
                                ui_counter = 0;
                                idx = 0;
                                idx1 = 0;
                                profile_editing_temp_profile = Profile::default();
                                changed = true;
                                profile_edit_state = ProfileEditSubMenus::default();
                                ui_state = UiState::MainMenu;
                            }

                            profile_edit_num = ui_counter;
                            profile_editing_temp_profile =
                                profiles.profiles[ui_counter as usize].unwrap_or_default();
                            ui_counter = 0;
                            profile_edit_state = ProfileEditSubMenus::ProfileElementSelect;
                            changed = true;
                        }
                    }
                    ProfileEditSubMenus::ProfileElementSelect => {
                        ui::edit_profile_menu(
                            &mut display,
                            ui_counter,
                            &profile[profile_edit_num as usize],
                            changed,
                        );
                        if changed {
                            changed = false;
                        }
                        if button {
                            match ui_counter {
                                0 => {
                                    profile_edit_state = ProfileEditSubMenus::ProfileNameEdit;
                                    changed = true;
                                }
                                1 => {
                                    profile_edit_state = ProfileEditSubMenus::ProfilePointSelect;
                                    changed = true;
                                }
                                2 => {
                                    profile_edit_state =
                                        ProfileEditSubMenus::ProfileWriteConfirmMenu;
                                    changed = true;
                                }
                                3 => {
                                    profile_edit_state =
                                        ProfileEditSubMenus::ProfileExitConfirmMenu;
                                    changed = true;
                                }
                                _ => {
                                    ui_counter = 0;
                                }
                            }
                            ui_counter = 0;
                        }
                    }
                    ProfileEditSubMenus::ProfileNameEdit => {
                        ui::edit_profile_name_menu(
                            &mut display,
                            ui_counter,
                            &profile[profile_edit_num as usize],
                            idx,
                            changed,
                        );
                        if changed {
                            changed = false;
                        }
                        if alt_button {
                            let char = ui::CHARACTERS[ui_counter as usize];
                            profile_editing_temp_profile.name[idx as usize] = char;
                            idx += 1;
                            if idx >= 6 {
                                idx = 0;
                            }
                            ui_counter = 0;
                        }

                        if button {
                            ui_counter = 0;
                            idx = 0;
                            changed = true;
                            profile_edit_state = ProfileEditSubMenus::ProfileElementSelect;
                        }
                    }
                    ProfileEditSubMenus::ProfilePointSelect => {
                        ui::edit_profile_points_select_menu(
                            &mut display,
                            ui_counter,
                            &profile_editing_temp_profile.points,
                            changed,
                        );
                        if changed {
                            changed = false;
                        }
                        if button {
                            match ui_counter {
                                0..=5 => {
                                    idx1 = ui_counter;
                                    changed = true;
                                    ui_counter = 0;
                                    profile_edit_state =
                                        ProfileEditSubMenus::ProfilePointSelectElementEdit;
                                }
                                6 => {
                                    idx1 = 0;
                                    changed = true;
                                    ui_counter = 0;
                                    profile_edit_state = ProfileEditSubMenus::ProfileElementSelect;
                                }
                                _ => {
                                    ui_counter = 0;
                                }
                            }
                            ui_counter = 0;
                        }
                    }
                    ProfileEditSubMenus::ProfilePointSelectElementEdit => {
                        ui::edit_profile_point_edit_select_menu(
                            &mut display,
                            ui_counter,
                            &profile_editing_temp_profile.points[idx1 as usize],
                            idx1,
                            changed,
                        );
                        if changed {
                            changed = false;
                        }
                        if button {
                            match ui_counter {
                                0 => {
                                    changed = true;
                                    ui_counter = 0;
                                    profile_edit_state = ProfileEditSubMenus::ProfilePointTempEdit;
                                }
                                1 => {
                                    changed = true;
                                    ui_counter = 0;
                                    profile_edit_state = ProfileEditSubMenus::ProfilePointTimeEdit;
                                }
                                2 => {
                                    changed = true;
                                    ui_counter = 0;
                                    profile_edit_state =
                                        ProfileEditSubMenus::ProfilePointDisabledEdit;
                                }
                                3 => {
                                    changed = true;
                                    ui_counter = 0;
                                    profile_edit_state = ProfileEditSubMenus::ProfilePointSelect;
                                }
                                _ => ui_counter = 0,
                            }
                            ui_counter = 0;
                        }
                    }
                    ProfileEditSubMenus::ProfilePointTempEdit => {
                        let rst = ui::edit_profile_point_edit_temp_menu(
                            &mut display,
                            ui_counter,
                            &profile_editing_temp_profile.points[idx1 as usize],
                            idx1,
                            changed,
                        );
                        if rst {
                            ui_counter = 0;
                        }
                        if changed {
                            changed = false;
                        }
                        if button {
                            profile_editing_temp_profile.points[idx as usize].temp =
                                ui_counter as u16;
                            ui_counter = 0;
                            changed = true;
                            profile_edit_state = ProfileEditSubMenus::ProfilePointSelectElementEdit;
                        }
                    }
                    ProfileEditSubMenus::ProfilePointTimeEdit => {
                        let rst = ui::edit_profile_point_edit_time_menu(
                            &mut display,
                            ui_counter,
                            &profile_editing_temp_profile.points[idx1 as usize],
                            idx1,
                            changed,
                        );
                        if rst {
                            ui_counter = 0;
                        }
                        if changed {
                            changed = false;
                        }
                        if button {
                            profile_editing_temp_profile.points[idx as usize].time_seconds =
                                ui_counter as u16;
                            ui_counter = 0;
                            changed = true;
                            profile_edit_state = ProfileEditSubMenus::ProfilePointSelectElementEdit;
                        }
                    }
                    ProfileEditSubMenus::ProfilePointDisabledEdit => {
                        let rst = ui::edit_profile_point_edit_disabled_menu(
                            &mut display,
                            ui_counter,
                            &profile_editing_temp_profile.points[idx1 as usize],
                            idx1,
                            direction == Direction::Clockwise,
                            changed,
                        );
                        if rst {
                            ui_counter = 0;
                        }
                        if changed {
                            changed = false;
                        }
                        if button {
                            profile_editing_temp_profile.points[idx as usize].disabled =
                                direction == Direction::Clockwise;
                            ui_counter = 0;
                            changed = true;
                            profile_edit_state = ProfileEditSubMenus::ProfilePointSelectElementEdit;
                        }
                    }
                    ProfileEditSubMenus::ProfileExitConfirmMenu => {
                        ui::edit_exit_menu(&mut display, ui_counter, changed);
                        if changed {
                            changed = false;
                        }
                        if button {
                            match ui_counter {
                                0 => {
                                    ui_counter = 0;
                                    changed = true;
                                    profile_edit_state = ProfileEditSubMenus::ProfileElementSelect;
                                }
                                1 => {
                                    idx = 0;
                                    idx1 = 0;
                                    profile_editing_temp_profile = Profile::default();
                                    ui_counter = 0;
                                    changed = true;
                                    profile_edit_state = ProfileEditSubMenus::ProfileSelect;
                                }
                                _ => {
                                    ui_counter = 0;
                                }
                            }
                        }
                    }
                    ProfileEditSubMenus::ProfileWriteConfirmMenu => {
                        let rst = ui::edit_save_exit_menu(&mut display, ui_counter, changed);
                        if rst {
                            ui_counter = 0;
                        }
                        if changed {
                            changed = false;
                        }
                        if button {
                            match ui_counter {
                                0 => {
                                    ui_counter = 0;
                                    changed = true;
                                    profile_edit_state = ProfileEditSubMenus::ProfileElementSelect;
                                }
                                1 => {
                                    idx = 0;
                                    idx1 = 0;
                                    profiles.profiles[idx as usize] =
                                        Some(profile_editing_temp_profile);
                                    profile_editing_temp_profile = Profile::default();
                                    ui_counter = 0;
                                    changed = true;
                                    profile_edit_state = ProfileEditSubMenus::ProfileSelect;
                                }
                                _ => {
                                    ui_counter = 0;
                                }
                            }
                        }
                    }
                },
                UiState::OvenRun => match oven_run_state {
                    OvenRunSubMenus::OvenProfileSelect => {
                        ui::heat_menu(&mut display, ui_counter, &profiles, changed);
                        if changed {
                            changed = false;
                        }
                        if button {
                            match ui_counter {
                                0..=15 => {
                                    if profiles.profiles[ui_counter as usize].is_none() {
                                        ui_counter = 0;
                                        run_profile_idx = 0;
                                        changed = true;
                                        ui_state = UiState::MainMenu;
                                        continue;
                                    }
                                    run_profile_idx = ui_counter;
                                    oven_run_state = OvenRunSubMenus::OvenProfileRunConfirm;
                                    changed = true;
                                    ui_counter = 0;
                                }
                                _ => {
                                    ui_counter = 0;
                                }
                            }
                        }
                    }
                    OvenRunSubMenus::OvenProfileRunConfirm => {
                        ui::start_heat_confirm_menu(
                            &mut display,
                            ui_counter,
                            &profiles.profiles[run_profile_idx as usize].unwrap(),
                            changed,
                        );
                        if changed {
                            changed = false;
                        }
                        if button {
                            match ui_counter {
                                0 => {
                                    oven_run_state = OvenRunSubMenus::OvenProfileSelect;
                                    ui_counter = 0;
                                    changed = true;
                                }
                                1 => {
                                    oven_run_state = OvenRunSubMenus::OvenProfileRunningMenu;
                                    ui_counter = 0;
                                    changed = true;
                                    running_oven = true;
                                    current_start_time = OYASUMI_TIME.load(MemOrdering::SeqCst);
                                }
                                _ => ui_counter = 0,
                            }
                        }
                    }
                    OvenRunSubMenus::OvenProfileRunningMenu => {
                        ui::heating_menu(
                            &mut display,
                            ui_counter,
                            temp,
                            &current_running_profile,
                            time_left as u16,
                            changed,
                        );
                        if changed {
                            changed = false;
                        }
                        if button {
                            oven_run_state = OvenRunSubMenus::OvenProfileCancelRunningMenu;
                        }
                    }
                    OvenRunSubMenus::OvenProfileCancelRunningMenu => {
                        let rst = ui::cancel_heat_menu(
                            &mut display,
                            ui_counter,
                            time_left as u16,
                            changed,
                        );
                        if changed {
                            changed = false;
                        }
                        if rst {
                            ui_counter = 0;
                        }
                        if button {
                            match ui_counter {
                                0 => {
                                    ui_counter = 0;
                                    oven_run_state = OvenRunSubMenus::OvenProfileRunningMenu;
                                }
                                1 => {
                                    ui_counter = 0;
                                    oven_run_state = OvenRunSubMenus::OvenProfileSelect;
                                    ui_state = UiState::MainMenu;
                                    running_oven = false;
                                    time_left = 0;
                                    current_start_time = 0;
                                }
                                _ => ui_counter = 0,
                            }
                        }
                    }
                },
            }
        }

        clocks += 1;
        delay_ms(1);
    }
}
