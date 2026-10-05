
#![feature(proc_macro_hygiene)]
#![feature(asm)]
#![allow(unused_imports)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(warnings, unused)]

use std::{fs, path::Path};

#[cfg(feature = "main_nro")]
use skyline_web::dialog_ok::DialogOk;

#[macro_use]
extern crate modular_bitfield;

#[macro_use]
extern crate lazy_static;

pub static mut FIGHTER_MANAGER: usize = 0;

use skyline::libc::c_char;
use skyline::nro::{self, NroInfo};
use smash::params::add_hook;
use std::sync::atomic::{AtomicBool, Ordering};
use skyline::hooks::InlineCtx;


pub fn is_on_ryujinx() -> bool {
    unsafe {
        // Ryujinx skip based on text addr
        let text_addr = skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) as u64;
        if text_addr == 0x8504000 || text_addr == 0x80004000 {
            println!("we are on Emulator");
            return true;
        } else {
            println!("we are not on Emulator");
            return false;
        }
    }
}
mod state_manager;
mod variable_module;
mod param_cache;
mod s_macros;
mod config;
mod config_apply;


pub fn quick_validate_install() -> bool {
    let has_param_config = Path::new(
        "rom:/skyline/plugins/libparam_config.nro",
    ).is_file();

    if has_param_config {
        println!("libparam_config.nro is present");
    } else {
        if is_on_ryujinx() {
            println!("libparam_config.nro not found! This installation is incomplete. Please run Ultimate S Setup Tool.");
        } else {
            DialogOk::ok("libparam_config.nro not found! This installation is incomplete. Please run Ultimate S Setup Tool.");
        }
        return false;
    }
    let has_css_redirector = Path::new(
        "rom:/skyline/plugins/libthe_csk_collection.nro",
    )
    .is_file();
    if has_css_redirector {
        println!("libthe_csk_collection.nro is present");
    } else {
        if is_on_ryujinx() {
            println!("libthe_csk_collection.nro not found! This installation is incomplete. Please run Ultimate S Setup Tool.");
        } else {
            DialogOk::ok("libthe_csk_collection.nro not found! This installation is incomplete. Please run Ultimate S Setup Tool.");
        }
        return false;
    }
    let has_arcropolis = Path::new(
        "rom:/skyline/plugins/libarcropolis.nro",
    )
    .is_file();
    if has_arcropolis {
        println!("libarcropolis.nro is present");
    } else {
        if is_on_ryujinx() {
            println!("libarcropolis.nro not found! This installation is incomplete. Please run Ultimate S Setup Tool.");
        } else {
            DialogOk::ok("libarcropolis.nro not found! This installation is incomplete. Please run Ultimate S Setup Tool.");
        }
        return false;
    }
    let has_nro_hook = Path::new(
        "rom:/skyline/plugins/libnro_hook.nro"
    )
    .is_file();
    if has_nro_hook {
        println!("libnro_hook.nro is present");
    } else {
        if is_on_ryujinx() {
            println!("libnro_hook.nro not found! This installation is incomplete. Please run Ultimate S Setup Tool.");
        } else {
            DialogOk::ok("libnro_hook.nro not found! This installation is incomplete. Please run Ultimate S Setup Tool.");
        }
        return false;
    }
    let has_smashline = Path::new(
        "rom:/skyline/plugins/libsmashline_plugin.nro",
    )
    .is_file();
    if has_smashline {
        println!("libsmashline_plugin.nro is present");
    } else {
        if is_on_ryujinx() {
            println!("libsmashline_plugin.nro not found! This installation is incomplete. Please run Ultimate S Setup Tool.");
        } else {
            DialogOk::ok("libsmashline_plugin.nro not found! This installation is incomplete. Please run Ultimate S Setup Tool.");
        }
        return false;
    }

    return true;
}

extern "C" {
	fn change_version_string(arg: u64, string: *const c_char);
}
pub fn nro_hook(info: &skyline::nro::NroInfo) {
    if info.module.isLoaded {
        return;
    }

    if info.name == "common" {
        skyline::install_hooks!(
            cpu::dmg_fly_main,
            cpu::dmg_fly_roll_main,
            cpu::dmg_main,
            cpu::dmg_air_main
        );
    }
}


unsafe fn calc_nnsdk_offset() -> u64 {
    let mut symbol = 0usize;
    skyline::nn::ro::LookupSymbol(&mut symbol, b"_ZN7android7IBinderD1Ev\0".as_ptr());
    (symbol - 0x240) as u64
}

static mut OFFSET1: u64 = 0;
static mut OFFSET2: u64 = 0;

#[skyline::hook(replace = OFFSET1)]
unsafe fn set_interval_1(window: u64, _: i32) {
    call_original!(window, 0);
}

#[skyline::hook(replace = OFFSET2, inline)]
unsafe fn set_interval_2(ctx: &mut InlineCtx) {
    ctx.registers[8].set_x(0);
    
}


static mut RUN: AtomicBool = AtomicBool::new(false);

#[skyline::hook(offset = 0x3810a64, inline)]
unsafe fn vsync_count_thread(_: &skyline::hooks::InlineCtx) {
    RUN.store(true, Ordering::SeqCst);
}

static mut DUMMY_BLOCK: [u8; 0x100] = [0; 0x100];

#[skyline::hook(offset = 0x3747b7c, inline)]
unsafe fn run_scene_update(_: &skyline::hooks::InlineCtx) {
    while !RUN.swap(false, Ordering::SeqCst) {
        skyline::nn::hid::GetNpadFullKeyState(DUMMY_BLOCK.as_mut_ptr() as _, &0);
    }
}
  
#[skyline::hook(replace = change_version_string)]
fn change_version_string_hook(arg: u64, string: *const c_char) {
	let original_str = unsafe { skyline::from_c_str(string) };
	if original_str.contains("Ver. 13") {
        if Path::new("sd:/ultimate/mods/Ultimate S Arcropolis/").is_dir() {
            let mut s_ver = match std::fs::read_to_string("sd:/ultimate/mods/Ultimate S Arcropolis/version.txt") {
                Ok(version_value) => version_value.trim().to_string(),
                Err(_) => {
                    String::from("UNKNOWN")
                }
            };
            let version_str = format!("{} / Ultimate S {}\0", original_str, s_ver);
            call_original!(arg, skyline::c_str(&version_str))
        } else {
            let mut s_ver = match std::fs::read_to_string("sd:/ultimate/mods/Ultimate S Lite/version.txt") {
                Ok(version_value) => version_value.trim().to_string(),
                Err(_) => {
                    String::from("UNKNOWN")
                }
            };
            let version_str = format!("{} / Ultimate S {}\0", original_str, s_ver);
            call_original!(arg, skyline::c_str(&version_str))
        }
	} else {
		call_original!(arg, string)
	}
}












mod util;
mod controls;
mod common;
mod cpu;

mod bayonetta;
mod bomberman;
mod brave;
mod buddy;
mod captain;
mod chrom;
mod cloud;
mod daisy;
mod dedede;
mod demon;
mod diddy;
mod dolly;
mod donkey;
mod duckhunt;
mod edge;
mod element;
mod falco;
mod fox;
mod gamewatch;
mod ganon;
mod gaogaen;
mod gekkouga;
mod ike;
mod inkling; 
mod jack;
mod kamui;
mod ken;
mod kirby;
mod koopa;
mod koopajr;
mod krool;
mod link;
mod littlemac;
mod lucario;
mod lucas;
mod lucina;
mod luigi;
mod mario;
mod mariod;
mod marth;
mod master;
mod metaknight;
mod mewtwo;
mod miifighter;
mod miigunner;
mod miiswordsman;
mod murabito;
mod ness;
mod packun;
mod pacman;
mod palutena;
mod peach;
mod peppy;
mod pichu;
mod pikachu;
mod pikmin;
mod pit;
mod pitb;
mod popo;
mod ptrainer;
mod purin;
mod rayman;
mod reflet;
mod richter;
mod ridley;
mod robot;
mod rockman;
mod rosetta;
mod roy;
mod ryu;
mod samus;
mod samusd;
mod sheik;
mod shizue;
mod shulk;
mod simon;
mod snake;
mod sonic;
mod szerosuit;
mod tantan;
mod toad;
mod toonlink;
mod trail;
mod wario;
mod wiifit;
mod wolf;
mod younglink;
mod yoshi;
mod zelda;

std::arch::global_asm!(
    r#"
    .section .nro_header
    .global __nro_header_start
    .word 0
    .word _mod_header
    .word 0
    .word 0
    
    .section .rodata.module_name
        .word 0
        .word 5
        .ascii "ultimate-s"
    .section .rodata.mod0
    .global _mod_header
    _mod_header:
        .ascii "MOD0"
        .word __dynamic_start - _mod_header
        .word __bss_start - _mod_header
        .word __bss_end - _mod_header
        .word __eh_frame_hdr_start - _mod_header
        .word __eh_frame_hdr_end - _mod_header
        .word __nx_module_runtime - _mod_header // runtime-generated module object offset
    .global IS_NRO
    IS_NRO:
        .word 1
    
    .section .bss.module_runtime
    __nx_module_runtime:
    .space 0xD0
    "#
);
#[no_mangle]
pub extern "C" fn is_ultimate_s() {}

#[no_mangle]
pub extern "C" fn main() {
    // ===== TOAD-ONLY BUILD =====
    // Everything except Toad (and the shared code Toad needs to run) has been
    // switched off. Removed compared to the full Ultimate S plugin:
    //  - the frame-timing ("less delay") hooks that made the game run too fast
    //  - the CSS hooks (Minus-button settings menu, game-mode menu on R)
    //  - Ultimate S's global mechanics changes (wavedash, hitstun, landing, etc.)
    //  - every other character's moveset changes, plus Rayman, Bomberman and Peppy
    //  - CPU behaviour changes, stage patches and the version-string change
    println!("[Toad only] Running config setup");
    match config::init() {
        Ok(_) => {
            println!("[Toad only] Config loaded successfully");
        }
        Err(e) => {
            if is_on_ryujinx() {
                println!("Your config.toml is incorrect! Please provide a correct one.");
            } else {
                DialogOk::ok("Your config.toml is incorrect! Please provide a correct one.");
            }
            return;
        }
    }
    if !quick_validate_install() {
        return; // don't do anything else since they don't have all dependencies
    }
    //allows online play with Toad
    unsafe {
        if Path::new("sd:/atmosphere/contents/01006a800016e000/romfs/skyline/plugins/libthe_csk_collection.nro").is_file() {
            extern "C" { fn allow_ui_chara_hash_online(ui_chara_hash: u64); }
            allow_ui_chara_hash_online(0xda4cbcb12); //toad
        }
    }
    // Shared code that Toad's moveset relies on
    util::install();
    cpu::install(); // only looks up the fighter manager, no CPU behaviour changes

    toad::install();
    println!("[Toad only] toad installed");

    param_cache::install();

    the_csk_collection_api::add_narration_characall_entry("vc_narration_characall_toad");
    the_csk_collection_api::add_narration_characall_entry("vc_narration_characall_toadette");
    the_csk_collection_api::add_narration_characall_entry("vc_narration_characall_toadsworth");
    the_csk_collection_api::add_narration_characall_entry("vc_narration_characall_captaintoad");
}
