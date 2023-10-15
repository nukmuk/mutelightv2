use std::ptr;
use std::ptr::{null, null_mut};
use std::thread::sleep;
use std::time::Duration;
use cue_sdk::device::CueDevice;
use cue_sdk::device::DeviceLayout::Keyboard;
use cue_sdk::led::{CueLed, LedColor};
use cue_sdk::led::KeyboardLedId::{KeyMute, KeyScanNextTrack, KeyWinLock};
use rdev::{Event, EventType, grab, Key};

use windows::Win32::{Media::Audio::*, System::Com::*};
use windows::Win32::Foundation::{FALSE, HWND, TRUE};
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolumeEx;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::{GetMessageA, MSG, WM_INPUT, WM_KEYFIRST};

fn main() {
    // register_hotkey(VK_F13);
    set_color(true);

    loop {};
}

fn set_color(muted: bool) {
    let cue = cue_sdk::initialize().unwrap();
    let mut devices = cue.get_all_devices().unwrap();

    let mut color;

    match muted {
        true => color = LedColor { red: 255, green: 0, blue: 0 },
        false => color = LedColor { red: 0, green: 255, blue: 0 },
    }

    let white = LedColor { red: 255, green: 255, blue: 255 };

    devices.iter_mut().for_each(|mut d: &mut CueDevice| {
        d.leds.get_mut(KeyMute as usize).unwrap().update_color_buffer(color).unwrap();
        cue.flush_led_colors_update_buffer_sync().unwrap();

        // d.leds.iter_mut().for_each(|mut l: &mut CueLed| {
        //     println!("{:?}", l);
        //     l.update_color_buffer(white).unwrap();
        //     cue.flush_led_colors_update_buffer_sync().unwrap();
        //     sleep(Duration::from_millis(100));
        // });
    });
}

fn register_hotkey(key: VIRTUAL_KEY) {
    unsafe {
        let hotkey = windows::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey(HWND(0), 1, MOD_NOREPEAT, key.0.into());
        println!("hotkey registered: {:?}", hotkey);

        CoInitialize(Some(null())).unwrap();

        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap();

        let mut msg = MSG::default();
        while GetMessageA(&mut msg, HWND(-1), 0, 0) != FALSE {
            println!("{:?}", msg);
            toggle_mute(&enumerator);
        };
    };
}

fn toggle_mute(enumerator: &IMMDeviceEnumerator) {
    unsafe {
        let device = enumerator
            .GetDefaultAudioEndpoint(eCapture, eMultimedia)
            .unwrap();

        let endpoint = device.Activate::<IAudioEndpointVolumeEx>(CLSCTX_ALL, Some(ptr::null())).unwrap();
        let muted = endpoint.GetMute().unwrap();
        match muted {
            TRUE => endpoint.SetMute(FALSE, ptr::null()).unwrap(),
            FALSE => endpoint.SetMute(TRUE, ptr::null()).unwrap(),
            _ => {}
        }
    }
}
/*
    unsafe {
        // CoInitializeEx(Some(ptr::null()), COINIT_MULTITHREADED).unwrap();

        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap();
        let device = enumerator
            .GetDefaultAudioEndpoint(eCapture, eMultimedia)
            .unwrap();

        let endpoint = device.Activate::<IAudioEndpointVolumeEx>(CLSCTX_ALL, Some(ptr::null())).unwrap();
        let muted = endpoint.GetMute().unwrap();
        match muted {
            TRUE => endpoint.SetMute(FALSE, ptr::null()).unwrap(),
            FALSE => endpoint.SetMute(TRUE, ptr::null()).unwrap(),
            _ => {}
        }

        println!("{:#?}", muted);
        // let manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, ptr::null()).unwrap();
        // let sessions = manager.GetSessionEnumerator().unwrap();
        //
        // for n in 0..sessions.GetCount().unwrap() {
        //     let session_control = sessions.GetSession(n).unwrap();
        // }

        // CoUninitialize();
    }
*/