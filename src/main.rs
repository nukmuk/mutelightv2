mod audio_device_volume_notification_client;

use audio_device_volume_notification_client::*;

use std::ptr;
use std::ptr::{null, null_mut};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use std::thread::sleep;
use std::time::Duration;
use cue_sdk::device::CueDevice;
use cue_sdk::device::DeviceLayout::Keyboard;
use cue_sdk::led::{CueLed, LedColor};
use cue_sdk::led::KeyboardLedId::{KeyMute, KeyScanNextTrack, KeyWinLock};
use rdev::{Event, EventType, grab, Key};

use windows::Win32::{Media::Audio::*, System::Com::*};
use windows::Win32::Foundation::{FALSE, HWND, TRUE};
use windows::Win32::Media::Audio::Endpoints::{IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl, IAudioEndpointVolumeEx};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::{GetMessageA, MSG, WM_INPUT, WM_KEYFIRST};
use log::{debug, error};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    set_color(false);
    // detect_mute();
    register_control_change_notify().await;
    // register_hotkey(VK_F13);

    // loop {};
}

fn set_color(muted: bool) {
    let mute_key = 84;
    let cue = cue_sdk::initialize().unwrap();
    let mut devices = cue.get_all_devices().unwrap();

    let mut color: LedColor;

    match muted {
        true => color = LedColor { red: 255, green: 0, blue: 0 },
        false => color = LedColor { red: 0, green: 255, blue: 0 },
    }

    devices.iter_mut().for_each(|mut d: &mut CueDevice| {
        d.leds.get_mut(mute_key).unwrap().update_color_buffer(color).unwrap();
        cue.flush_led_colors_update_buffer_sync().unwrap();
    });
}

unsafe fn get_audio_enumerator() -> IMMDeviceEnumerator {
    CoInitialize(Some(null())).unwrap();

    CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap()
}

unsafe fn get_default_input_endpoint(enumerator: &IMMDeviceEnumerator) -> IAudioEndpointVolumeEx {
    let device = enumerator
        .GetDefaultAudioEndpoint(eCapture, eMultimedia)
        .unwrap();

    device.Activate::<IAudioEndpointVolumeEx>(CLSCTX_ALL, Some(ptr::null())).unwrap()
}

async fn register_control_change_notify() {
    unsafe {
        let enumerator = get_audio_enumerator();
        let endpoint = get_default_input_endpoint(&enumerator);

        let (on_notify_tx, mut on_notify_rx) = channel(10);

        let notification_client = AudioDeviceVolumeNotificationClient::new(on_notify_tx);

        endpoint.RegisterControlChangeNotify(&notification_client).unwrap();

        while let Some(msg) = on_notify_rx.recv().await {
            println!("received control change notify: {:?}", msg);
            let muted = endpoint.GetMute().unwrap();
            match muted {
                TRUE => set_color(true),
                FALSE => set_color(false),
                _ => {}
            };
        }


        println!("unregistering control change notify");

        endpoint.UnregisterControlChangeNotify(&notification_client).unwrap();
    }
}

fn register_hotkey(key: VIRTUAL_KEY) {
    unsafe {
        let hotkey = windows::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey(HWND(0), 1, MOD_NOREPEAT, key.0.into());
        println!("hotkey registered: {:?}", hotkey);

        let enumerator = get_audio_enumerator();

        let mut msg = MSG::default();
        while GetMessageA(&mut msg, HWND(-1), 0, 0) != FALSE {
            println!("{:?}", msg);
            toggle_mute(&enumerator);
        };
    };
}

fn toggle_mute(enumerator: &IMMDeviceEnumerator) {
    unsafe {
        let endpoint = get_default_input_endpoint(enumerator);

        let muted = endpoint.GetMute().unwrap();
        match muted {
            TRUE => endpoint.SetMute(FALSE, ptr::null()).unwrap(),
            FALSE => endpoint.SetMute(TRUE, ptr::null()).unwrap(),
            _ => {}
        }
    }
}

/*

maybe old code idk:

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