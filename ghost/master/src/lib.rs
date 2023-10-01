mod autolinefeed;
mod events;
mod roulette;
mod status;
mod variables;

use crate::variables::get_global_vars;

use std::fs::File;
use std::path::Path;
use std::slice;

use shiorust::message::*;

use winapi::ctypes::c_long;
use winapi::shared::minwindef::{BOOL, DWORD, HGLOBAL, HINSTANCE, LPBOOL, LPVOID, MAX_PATH, TRUE};
use winapi::shared::ntdef::{LPCSTR, NULL};
use winapi::um::libloaderapi::GetModuleFileNameW;
use winapi::um::stringapiset::WideCharToMultiByte;
use winapi::um::winbase::{GlobalAlloc, GlobalFree, GMEM_FIXED};
use winapi::um::winnt::{
    DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH, LPSTR,
};

#[macro_use]
extern crate log;
extern crate simplelog;

use simplelog::*;

static mut DLL_PATH: String = String::new();

#[derive(Debug)]
pub enum ResponseError {
    DecodeFailed,
}

#[no_mangle]
pub extern "system" fn DllMain(
    h_module: HINSTANCE,
    ul_reason_for_call: DWORD,
    _l_reserved: LPVOID,
) -> BOOL {
    match ul_reason_for_call {
        DLL_PROCESS_ATTACH => {
            register_dll_path(h_module);
            let path;
            unsafe {
                path = Path::new(&DLL_PATH.clone())
                    .parent()
                    .unwrap()
                    .join("haine.log");
            };
            WriteLogger::init(
                LevelFilter::Debug,
                Config::default(),
                File::create(path).unwrap(),
            )
            .unwrap();
            debug!("DLL_PROCESS_ATTACH");
        }
        DLL_PROCESS_DETACH => {
            debug!("DLL_PROCESS_DETACH");
        }
        DLL_THREAD_ATTACH => {}
        DLL_THREAD_DETACH => {
            debug!("DLL_THREAD_DETACH");
        }
        _ => {}
    }
    return TRUE;
}

fn register_dll_path(h_module: HINSTANCE) {
    let mut buf: [u16; MAX_PATH + 1] = [0; MAX_PATH + 1];
    unsafe {
        GetModuleFileNameW(h_module, buf.as_mut_ptr(), MAX_PATH as u32);
    }

    let p = buf.partition_point(|v| *v != 0);

    unsafe {
        DLL_PATH = String::from_utf16_lossy(&buf[..p]);
    }
}

#[no_mangle]
pub extern "cdecl" fn load(h: HGLOBAL, _len: c_long) -> BOOL {
    unsafe { GlobalFree(h) };

    debug!("load");

    get_global_vars().load();

    return TRUE;
}

#[no_mangle]
pub extern "cdecl" fn unload() -> BOOL {
    debug!("unload");

    get_global_vars().save();

    return TRUE;
}

#[no_mangle]
pub extern "cdecl" fn request(h: HGLOBAL, len: *mut c_long) -> HGLOBAL {
    // リクエストの取得
    let v = unsafe { hglobal_to_vec_u8(h, *len) };
    unsafe { GlobalFree(h) };

    let s = String::from_utf8_lossy(&v).to_string();

    let r = Request::parse(&s).unwrap();

    let response = events::handle_request(&r);

    let response_bytes = to_encoded_bytes(response).unwrap_or_else(|e| {
        debug!("error: {:?}", e);
        vec![]
    });

    let h = slice_i8_to_hglobal(len, &response_bytes);

    h
}

fn to_encoded_bytes(res: Response) -> Result<Vec<i8>, ResponseError> {
    let req = res.to_string();

    let mut wide_chars: Vec<u16> = req.encode_utf16().collect();

    const UTF8: u32 = 65001;
    let result =
        wide_char_to_multi_byte(&mut wide_chars, UTF8).map_err(|_| ResponseError::DecodeFailed)?;

    Ok(result)
}

fn wide_char_to_multi_byte(from: &mut Vec<u16>, codepage: u32) -> Result<Vec<i8>, ()> {
    from.push(0);

    let to_buf_size = unsafe {
        WideCharToMultiByte(
            codepage,
            0,
            from.as_ptr(),
            -1,
            NULL as LPSTR,
            0,
            NULL as LPCSTR,
            NULL as LPBOOL,
        )
    };

    if to_buf_size == 0 {
        return Err(());
    }

    let mut to_buf: Vec<i8> = vec![0; to_buf_size as usize + 1];
    let result = unsafe {
        WideCharToMultiByte(
            codepage,
            0,
            from.as_ptr(),
            -1,
            to_buf.as_mut_ptr(),
            to_buf_size,
            NULL as LPCSTR,
            NULL as LPBOOL,
        )
    };

    if result == 0 {
        Err(())
    } else {
        Ok(to_buf)
    }
}

fn slice_i8_to_hglobal(h_len: *mut c_long, data: &[i8]) -> HGLOBAL {
    let data_len = data.len();

    let h = unsafe { GlobalAlloc(GMEM_FIXED, data_len) };

    unsafe { *h_len = data_len as c_long };

    let h_slice = unsafe { slice::from_raw_parts_mut(h as *mut i8, data_len) };

    for (index, value) in data.iter().enumerate() {
        h_slice[index] = *value;
    }

    return h;
}

fn hglobal_to_vec_u8(h: HGLOBAL, len: c_long) -> Vec<u8> {
    let mut s = vec![0; len as usize + 1];

    let slice = unsafe { slice::from_raw_parts(h as *const u8, len as usize) };

    for (index, value) in slice.iter().enumerate() {
        s[index] = *value;
    }
    s[len as usize] = b'\0';

    return s;
}
