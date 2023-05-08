use crate::profile::{CurvePoint, Profile, Profiles};
use core::str::from_utf8_unchecked;
use lcd::{Delay, Display, Hardware};

pub fn main_menu<T: Hardware + Delay>(
    hw: &mut Display<T>,
    temp: u16,
    counter: u8,
    cont: bool,
) -> bool {
    if !cont {
        hw.clear();
        writeln!(hw, "TEMP: {}", temp).unwrap();
    }
    hw.position(0, 1);
    match counter {
        0 => write!(hw, "*0: HEAT 1:EDIT ").unwrap(),
        1 => write!(hw, "0: HEAT *1:EDIT ").unwrap(),
        2 => write!(hw, "1:EDIT *2:CONFIG").unwrap(),
        _ => true,
    }
    false
}

pub fn heat_menu<T: Hardware + Delay>(
    hw: &mut Display<T>,
    counter: u8,
    profiles: &Profiles,
    cont: bool,
) -> bool {
    if !cont {
        hw.clear();
        writeln!(hw, "CHOOSE PROF:").unwrap();
    }
    hw.position(0, 1);
    match counter {
        p => match profiles.profiles[p as usize] {
            Some(prof) => {
                write!(hw, "*{}: {}", p, unsafe { from_utf8_unchecked(&prof.name) }).unwrap();
            }
            None => {
                write!(hw, "* GO BACK.").unwrap();
                return false;
            }
        },
    }
    true
}

pub fn start_heat_confirm_menu<T: Hardware + Delay>(
    hw: &mut Display<T>,
    counter: u8,
    profile: &Profile,
    cont: bool,
) -> bool {
    if !cont {
        hw.clear();
        writeln!(hw, "PROFILE {}:", unsafe {
            from_utf8_unchecked(&profile.name)
        })
        .unwrap();
    }
    hw.position(0, 1);
    match counter {
        0 => {
            write!(hw, "SURE?: * NO | YES").unwrap();
        }
        1 => {
            write!(hw, "SURE?: NO | * YES").unwrap();
        }
        _ => false,
    }
    true
}

pub fn heating_menu<T: Hardware + Delay>(
    hw: &mut Display<T>,
    counter: u8,
    temp: u16,
    profile: &Profile,
    time_left: u16,
    cont: bool,
) -> bool {
    if !cont {
        hw.clear();
        writeln!(hw, "RUN: {}", unsafe { from_utf8_unchecked(&profile.name) }).unwrap();
    }
    hw.position(0, 1);
    match counter {
        _ => {
            write!("{}C, {}LEFT", temp, time_left);
            false
        }
    }
}

pub fn cancel_heat_menu<T: Hardware + Delay>(
    hw: &mut Display<T>,
    counter: u8,
    time_left: u16,
    cont: bool,
) -> bool {
    if !cont {
        hw.clear();
        writeln!(hw, "CANCEL?").unwrap();
    }
    hw.position(0, 1);
    match counter {
        0 => {
            write!(hw, "SURE?: * NO | YES").unwrap();
        }
        1 => {
            write!(hw, "SURE?: NO | * YES").unwrap();
        }
        _ => false,
    }
    true
}

pub fn select_edit_profile_menu<T: Hardware + Delay>(
    hw: &mut Display<T>,
    counter: u8,
    profiles: &Profiles,
    cont: bool,
) -> bool {
    if !cont {
        hw.clear();
        writeln!(hw, "EDIT PROF:").unwrap();
    }
    hw.position(0, 1);
    match counter {
        p => {
            if p >= 16 {
                write!(hw, "*GO BACK.").unwrap();
                return false;
            }
            match profiles.profiles[p as usize] {
                Some(prof) => {
                    write!(hw, "*{}: {}", p, unsafe { from_utf8_unchecked(&prof.name) }).unwrap();
                }
                None => {
                    write!(hw, "* <EMPTY SLOT>").unwrap();
                }
            }
        }
    }
    true
}

pub fn edit_profile_menu<T: Hardware + Delay>(
    hw: &mut Display<T>,
    counter: u8,
    profile: &Profile,
    cont: bool,
) -> bool {
    if !cont {
        hw.clear();
        writeln!(hw, "EDIT {}:", unsafe {
            from_utf8_unchecked(&profile.name)
        })
        .unwrap();
    }
    hw.position(0, 1);
    match counter {
        0 => {
            write!(hw, "*0:NAME 1:TEMP").unwrap();
        }
        1 => {
            write!(hw, "*1:TEMP 2:SAVE").unwrap();
        }
        2 => {
            write!(hw, "*2:SAVE 3:EXIT").unwrap();
        }
        3 => {
            write!(hw, "*3:EXIT 0:NAME").unwrap();
        }
        _ => false,
    }
    true
}

pub const CHARACTERS: [u8; 36] = [
    b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p',
    b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'1', b'2', b'3', b'4', b'5', b'6',
    b'7', b'8', b'9', b'0',
];

pub fn edit_profile_name_menu<T: Hardware + Delay>(
    hw: &mut Display<T>,
    counter: u8,
    words: &[u8; 6],
    idx: u8,
    cont: bool,
) -> bool {
    if !cont {
        hw.clear();
        writeln!(hw, "NAME EDIT {}:", unsafe {
            from_utf8_unchecked(&profile.name)
        })
        .unwrap();
    }
    hw.position(0, 1);
    match counter {
        cnt => {
            if cnt > 35 {
                return false;
            }
            let mut wd2 = words.clone();
            wd2[idx] = CHARACTERS[cont];
            writeln!(hw, "{}", unsafe { from_utf8_unchecked(&wd2) }).unwrap();
            true
        }
    }
}

pub fn edit_profile_points_select_menu<T: Hardware + Delay>(
    hw: &mut Display<T>,
    counter: u8,
    points: &[CurvePoint; 6],
    cont: bool,
) -> bool {
    if !cont {
        hw.clear();
        writeln!(hw, "POINT SELECT").unwrap();
    }
    hw.position(0, 1);
    match counter {
        pt => {
            if pt > 6 {
                return false;
            } else if pt == 6 {
                write!(hw, "*RETURN").unwrap();
            }
            let p = points.get(pt as usize).unwrap();
            write!(
                hw,
                "*{}: {} {} {}",
                counter, p.temp, p.time_seconds, p.disabled
            )
            .unwrap();
        }
    }
    true
}

pub fn edit_profile_point_edit_select_menu<T: Hardware + Delay>(
    hw: &mut Display<T>,
    counter: u8,
    point: &CurvePoint,
    idx: u8,
    cont: bool,
) -> bool {
    if !cont {
        hw.clear();
        writeln!(hw, "POINT {}", idx).unwrap();
    }
    hw.position(0, 1);
    match counter {
        0 => {
            write!(hw, "*01: TEMP {}", point.temp).unwrap();
        }
        1 => {
            write!(hw, "*02: TIME {}", point.temp).unwrap();
        }
        2 => {
            write!(hw, "*03: DISABLED {}", point.temp).unwrap();
        }
        3 => {
            write!(hw, "*04: GO BACK").unwrap();
        }
        _ => false,
    }
    true
}

pub fn edit_profile_point_edit_temp_menu<T: Hardware + Delay>(
    hw: &mut Display<T>,
    counter: u8,
    point: &CurvePoint,
    idx: u8,
    cont: bool,
) -> bool {
    if !cont {
        hw.clear();
        writeln!(hw, "PT-{}, TEMP {}", idx, point.temp).unwrap();
    }
    hw.position(0, 1);
    match counter {
        cnt => {
            if cnt == u8::MAX {
                return false;
            }
            write!(hw, "{} DEG CEL", cnt).unwrap();
        }
    }
    true
}

pub fn edit_profile_point_edit_time_menu<T: Hardware + Delay>(
    hw: &mut Display<T>,
    counter: u8,
    point: &CurvePoint,
    idx: u8,
    cont: bool,
) -> bool {
    if !cont {
        hw.clear();
        writeln!(hw, "PT-{}, TIME {}", idx, point.time_seconds).unwrap();
    }
    hw.position(0, 1);
    match counter {
        cnt => {
            if cnt == u8::MAX {
                return false;
            }
            write!(hw, "{} SECONDS", cnt).unwrap();
        }
    }
    true
}

pub fn edit_profile_point_edit_disabled_menu<T: Hardware + Delay>(
    hw: &mut Display<T>,
    counter: u8,
    point: &CurvePoint,
    idx: u8,
    state: bool,
    cont: bool,
) -> bool {
    if !cont {
        hw.clear();
        writeln!(hw, "PT-{}, DISABLED {}", idx, point.disabled).unwrap();
    }
    hw.position(0, 1);
    match counter {
        cnt => {
            if cnt == u8::MAX {
                return false;
            }
            write!(hw, "DISABLED: {}", state).unwrap();
        }
    }
    true
}

pub fn edit_exit_menu<T: Hardware + Delay>(hw: &mut Display<T>, counter: u8, cont: bool) -> bool {
    if !cont {
        hw.clear();
        writeln!(hw, "EXIT?").unwrap();
    }
    hw.position(0, 1);
    match counter {
        0 => {
            write!(hw, "SURE?: * NO | YES").unwrap();
        }
        1 => {
            write!(hw, "SURE?: NO | * YES").unwrap();
        }
        _ => false,
    }
    true
}

pub fn edit_save_exit_menu<T: Hardware + Delay>(
    hw: &mut Display<T>,
    counter: u8,
    cont: bool,
) -> bool {
    if !cont {
        hw.clear();
        writeln!(hw, "SAVE & EXIT?").unwrap();
    }
    hw.position(0, 1);
    match counter {
        0 => {
            write!(hw, "SURE?: * NO | YES").unwrap();
        }
        1 => {
            write!(hw, "SURE?: NO | * YES").unwrap();
        }
        _ => false,
    }
    true
}
