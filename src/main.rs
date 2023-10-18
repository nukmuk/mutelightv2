mod audio_device_volume_notification_client;
mod device_changed_notification_client;

use audio_device_volume_notification_client::*;

use std::{ptr, thread};
use std::ops::Deref;
use std::ptr::{NonNull, null, null_mut};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{park, sleep};
use std::time::Duration;
use cue_sdk::device::CueDevice;
use cue_sdk::device::DeviceLayout::Keyboard;
use cue_sdk::led::{CueLed, LedColor};
use cue_sdk::led::KeyboardLedId::{KeyMute, KeyScanNextTrack, KeyWinLock};
use rdev::{Event, EventType, grab, Key};

use windows::Win32::{Media::Audio::*, System::Com::*};
use windows::Win32::Foundation::{BOOL, FALSE, HWND, TRUE};
use windows::Win32::Media::Audio::Endpoints::{IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl, IAudioEndpointVolumeEx};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::{GetMessageA, MSG, WM_INPUT, WM_KEYFIRST};
use log::{debug, error};
use tokio::sync::mpsc;
use windows::core::{ComInterface, Interface};
use crate::device_changed_notification_client::DeviceChangedNotificationClient;

use windows::Win32::System::WinRT::{RO_INIT_TYPE, RoInitialize};

#[derive(Debug)]
struct DeviceEnumerator(IMMDeviceEnumerator);

impl DeviceEnumerator {
    // fn new(enumerator: IMMDeviceEnumerator) -> Self {
    //     Self(NonNull::new(enumerator).unwrap())
    // }
}

impl From<IMMDeviceEnumerator> for DeviceEnumerator {
    fn from(enumerator: IMMDeviceEnumerator) -> Self {
        DeviceEnumerator(enumerator)
    }
}

unsafe impl Send for DeviceEnumerator {}

#[tokio::main]
async fn main() {
    unsafe {
        // getting multiple enumerators always results in same pointer

        let original_enumerator = Arc::new(Mutex::new(get_audio_enumerator().into())).clone();
        let enumerator = original_enumerator.clone();


        let devicechanged_handle = thread::spawn(move || {
            detect_device_changed(enumerator);
            println!("device_handle finished");
        });

        let enumerator = original_enumerator.clone();
        let ismuted_handle = thread::spawn(move || {
            let endpoint = register_devicestate_change_notify(enumerator);
            println!("state_handle finished");
            park();
        });

        let enumerator = original_enumerator.clone();
        let hotkey_handle = thread::spawn(move || {
            register_hotkey(VK_INSERT, enumerator);
            println!("hotkey_handle finished");
        });

        // wait for all threads to finish
        devicechanged_handle.join().unwrap();
        ismuted_handle.join().unwrap();
        hotkey_handle.join().unwrap();

        park();
    }
}

fn detect_device_changed(enumerator: Arc<Mutex<DeviceEnumerator>>) {
    unsafe {
        let notification_client = DeviceChangedNotificationClient::new();
        let enumerator: &IMMDeviceEnumerator = &enumerator.lock().unwrap().0;

        enumerator.RegisterEndpointNotificationCallback(&notification_client).unwrap();
    }
}

fn set_color(muted: BOOL) {
    let mute_key = 84;
    let cue = cue_sdk::initialize().unwrap();
    let mut devices = cue.get_all_devices().unwrap();

    let color = match muted {
        TRUE => LedColor { red: 255, green: 0, blue: 0 },
        FALSE => LedColor { red: 0, green: 255, blue: 0 },
        _ => LedColor { red: 255, green: 255, blue: 0 },
    };

    dbg!("changing color to {}", color);

    devices.iter_mut().for_each(|mut d: &mut CueDevice| {
        d.leds.get_mut(mute_key).unwrap().update_color_buffer(color).unwrap();
        cue.flush_led_colors_update_buffer_sync().unwrap();
    });
}

unsafe fn get_audio_enumerator() -> IMMDeviceEnumerator {
    CoInitializeEx(Some(null_mut()), COINIT_MULTITHREADED).unwrap();
    // RoInitialize(RO_INIT_TYPE(0)).unwrap();
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap();
    dbg!(&enumerator);
    enumerator
}

unsafe fn get_default_input_endpoint(enumerator: &IMMDeviceEnumerator) -> IAudioEndpointVolumeEx {
    let device = enumerator
        .GetDefaultAudioEndpoint(eCapture, eMultimedia)
        .unwrap();

    device.Activate::<IAudioEndpointVolumeEx>(CLSCTX_ALL, Some(ptr::null())).unwrap()
}

fn register_devicestate_change_notify(enumerator: Arc<Mutex<DeviceEnumerator>>) -> IAudioEndpointVolumeEx {
    unsafe {
        let deviceenumerator = &enumerator.lock().unwrap().0;
        let endpoint = get_default_input_endpoint(&deviceenumerator);

        // let (on_notify_tx, mut on_notify_rx) = ch/annel(10);

        let notification_client = AudioDeviceVolumeNotificationClient::new();

        // if endpoint gets dropped it will unregister the notification client
        endpoint.RegisterControlChangeNotify(&notification_client).unwrap();

        endpoint
    }
}

fn register_hotkey(key: VIRTUAL_KEY, enumerator: Arc<Mutex<DeviceEnumerator>>) {
    unsafe {
        let hotkey = windows::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey(HWND(0), 1, MOD_NOREPEAT, key.0.into());
        println!("hotkey registered: {:?}", hotkey);

        let mute_enumerator = enumerator.clone();

        let mut msg = MSG::default();
        while GetMessageA(&mut msg, HWND(-1), 0, 0) != FALSE {
            println!("{:?}", msg);
            toggle_mute(mute_enumerator.clone());
        };
    };
}

fn toggle_mute(enumerator: Arc<Mutex<DeviceEnumerator>>) {
    unsafe {
        let endpoint = get_default_input_endpoint(&enumerator.lock().unwrap().0);

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