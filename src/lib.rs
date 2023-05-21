mod bindings;

use bindings as ffi;

use std::ffi::{c_char, CString, CStr};

pub struct VoiceVoxCore;

#[derive(Debug, Clone)]
pub enum AccelationMode {
    None,
    Cpu,
    Gpu,
}

impl Into<i32> for AccelationMode {
    fn into(self) -> i32 {
        match self {
            AccelationMode::None => 0,
            AccelationMode::Cpu => 1,
            AccelationMode::Gpu => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CpuNumThreads {
    Auto,
    Num(u16),
}

impl Into<u16> for CpuNumThreads {
    fn into(self) -> u16 {
        match self {
            CpuNumThreads::Auto => 0,
            CpuNumThreads::Num(n) => n,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InitializeOptions {
    pub acceleration_mode: AccelationMode,
    pub cpu_num_threads: CpuNumThreads,
    pub load_all_models: bool,
    pub open_jtalk_dict_dir: String,
}

impl Default for InitializeOptions {
    fn default() -> Self {
        InitializeOptions {
            acceleration_mode: AccelationMode::None,
            cpu_num_threads: CpuNumThreads::Auto,
            load_all_models: false,
            open_jtalk_dict_dir: "open_jtalk_dic_utf_8-1.11".into(),
        }
    }
}

impl VoiceVoxCore {
    pub fn new() -> Self {
        VoiceVoxCore {}
    }

    pub fn initialize(options: InitializeOptions) -> Result<(), String> {
        let open_jtalk = CString::new(options.open_jtalk_dict_dir.as_bytes()).unwrap();
        let result = unsafe {
            ffi::voicevox_initialize(crate::ffi::VoicevoxInitializeOptions {
                acceleration_mode: options.acceleration_mode.into(),
                cpu_num_threads: options.cpu_num_threads.into(),
                load_all_models: options.load_all_models,
                open_jtalk_dict_dir: open_jtalk.as_ptr(),
            })
        };

        if result != 0 {
            return Err("failed to initialize".into());
        };

        Ok(())
    }

    pub fn load_model(speaker_id: u32) -> Result<(), String> {
        let result = unsafe { crate::ffi::voicevox_load_model(speaker_id) };

        if result != 0 {
            Err("failed to load model".into())
        } else {
            Ok(())
        }
    }

    pub fn new_with_initialize(options: InitializeOptions) -> Result<Self, String> {
        let open_jtalk = CString::new(options.open_jtalk_dict_dir.as_bytes()).unwrap();
        let result = unsafe {
            ffi::voicevox_initialize(crate::ffi::VoicevoxInitializeOptions {
                acceleration_mode: options.acceleration_mode.into(),
                cpu_num_threads: options.cpu_num_threads.into(),
                load_all_models: options.load_all_models,
                open_jtalk_dict_dir: open_jtalk.as_ptr(),
            })
        };

        if result != 0 {
            return Err("failed to initialize".into());
        };

        Ok(VoiceVoxCore {})
    }

    pub fn audio_query(&mut self, text: &str, speaker_id: u32) -> Result<String, String> {
        let text = CString::new(text).unwrap();

        let mut raw_ptr: *mut c_char = std::ptr::null_mut();

        let result = unsafe {
            crate::ffi::voicevox_audio_query(
                text.as_ptr(),
                speaker_id,
                crate::ffi::VoicevoxAudioQueryOptions { kana: false },
                &mut raw_ptr,
            )
        };

        if result != 0 || raw_ptr.is_null() {
            return Err("failed to query".into());
        };

        let json = unsafe { CStr::from_ptr(raw_ptr) };
        let json = json.to_str();
        let Ok(json) = json else {
            unsafe { ffi::voicevox_audio_query_json_free(raw_ptr) };
            return Err("failed to parse json".into())
        };

        let json = json.to_string();
        unsafe { ffi::voicevox_audio_query_json_free(raw_ptr) };


        Ok(json)
    }

    pub fn synthesis(&mut self, query: String) -> Result<Vec<u8>, String> {
        let mut out_len: usize = 0;

        let query = CString::new(query).unwrap();

        let mut wav: *mut u8 = std::ptr::null_mut();

        unsafe {
            let result = crate::ffi::voicevox_synthesis(
                query.as_ptr(),
                1,
                crate::ffi::VoicevoxSynthesisOptions {
                    enable_interrogative_upspeak: false,
                },
                &mut out_len,
                &mut wav,
            );

            if result != 0 {
                return Err(format!("failed to synthesis {}", result));
            };

            let bytes = if !wav.is_null() && out_len > 0 {
                let bytes = std::slice::from_raw_parts(wav, out_len).to_vec();
                ffi::voicevox_wav_free(wav);
                bytes
            } else {
                vec![]
            };

            if bytes.is_empty() {
                return Err("empty bytes".into());
            };

            Ok(bytes)
        }
    }

    pub fn finalize() {
        unsafe {
            ffi::voicevox_finalize();
        }
    }
}


// impl Drop for VoiceVoxCore {
//     fn drop(&mut self) {
//         unsafe {
//             ffi::voicevox_finalize();
//         }
//     }
// }