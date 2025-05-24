type Result<T> = color_eyre::Result<T>;

fn main() -> crate::Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        return Err(color_eyre::eyre::eyre!("Requires Windows"));
    }

    #[cfg(target_os = "windows")]
    {
        use notify_rust::{Notification, Timeout};
        use windows::Win32::{
            Media::Audio::{
                DEVICE_STATE_ACTIVE, Endpoints::IAudioEndpointVolume, IMMDevice,
                IMMDeviceEnumerator, MMDeviceEnumerator, eCapture,
            },
            System::Com::{CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx},
        };

        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            // Create the device enumerator
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

            let collection = enumerator.EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)?;

            let count = collection.GetCount()?;
            if count == 0 {
                Notification::new()
                    .summary("Microphone")
                    .body("No active microphones found")
                    .timeout(Timeout::Milliseconds(5))
                    .show()?;
                return Ok(());
            }

            let first_device: IMMDevice = collection.Item(0)?;

            let first_endpoint_volume: IAudioEndpointVolume =
                first_device.Activate(CLSCTX_ALL, None)?;

            let is_muted = first_endpoint_volume.GetMute()?.as_bool();

            // Apply the new mute state to all devices
            for i in 0..count {
                // Get the audio endpoint device
                let device: IMMDevice = collection.Item(i)?;
                let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;

                endpoint_volume.SetMute(!is_muted, std::ptr::null_mut())?;
            }

            let notification_body = if !is_muted {
                "All microphones muted"
            } else {
                "All microphones unmuted"
            };

            Notification::new()
                .summary("Microphone")
                .body(notification_body)
                .timeout(Timeout::Milliseconds(1))
                .id(1)
                .show()?;
        }
    }

    Ok(())
}
