#[macro_use]
mod events;

mod autobreakline;
mod error;
mod roulette;
mod status;
mod variables;

use crate::events::common::{add_error_description, new_response_nocontent};
use crate::variables::*;

use std::fs::{metadata, File};
use std::panic;
use std::path::PathBuf;

use shiori_hglobal::*;
use shiorust::message::*;

use winapi::ctypes::c_long;
use winapi::shared::minwindef::{BOOL, FALSE, HGLOBAL, TRUE};

#[macro_use]
extern crate log;
extern crate simplelog;

use simplelog::*;

#[no_mangle]
pub extern "cdecl" fn loadu(h: HGLOBAL, len: c_long) -> BOOL {
  let v = GStr::capture(h, len as usize);
  let s = match v.to_utf8_str() {
    Ok(st) => {
      // UTF-8に変換
      st.to_string()
    }
    Err(e) => {
      eprintln!("Failed to convert HGLOBAL to UTF-8: {:?}", e);
      return FALSE;
    }
  };
  match common_load_procedure(&s) {
    Ok(_) => {
      debug!("loadu");
      TRUE
    }
    Err(_) => {
      error!("error while loading");
      FALSE
    }
  }
}

#[no_mangle]
pub extern "cdecl" fn load(h: HGLOBAL, len: c_long) -> BOOL {
  let v = GStr::capture(h, len as usize);
  let s: String;
  match v.to_utf8_str() {
    Ok(st) => {
      // UTF-8に変換
      s = st.to_string();
    }
    Err(e) => {
      eprintln!("Failed to convert HGLOBAL to UTF-8: {:?}", e);
      match v.to_ansi_str() {
        Ok(st) => {
          // ANSIに変換
          s = st.to_string_lossy().to_string();
        }
        Err(e) => {
          eprintln!("Failed to convert HGLOBAL to ANSI: {:?}", e);
          return FALSE;
        }
      }
    }
  };

  match common_load_procedure(&s) {
    Ok(_) => {
      debug!("load");
      TRUE
    }
    Err(_) => {
      error!("error while loading");
      FALSE
    }
  }
}

fn common_load_procedure(path: &str) -> Result<(), ()> {
  // ログの設定
  // Windows(UTF-16)を想定しPathBufでパスを作成
  let log_path = PathBuf::from(path).join("haine.log");
  *LOG_PATH.write().unwrap() = log_path.to_str().unwrap().to_string();
  let fp = if let Ok(fp) = File::create(log_path) {
    fp
  } else {
    error!("error while creating log file");
    return Err(());
  };
  match WriteLogger::init(LevelFilter::Debug, Config::default(), fp) {
    Ok(_) => {}
    Err(e) => {
      error!("{}", e);
      return Err(());
    }
  }

  panic::set_hook(Box::new(|panic_info| {
    debug!("{}", panic_info);
  }));

  if let Err(e) = load_global_variables() {
    error!("{}", e);
  }

  // ./debugが存在するならデバッグモード
  if metadata("./debug").is_ok() {
    *DEBUG_MODE.write().unwrap() = true;
  }

  // Inserterの初期化を別スレッドで開始
  INSERTER.write().unwrap().start_init();

  Ok(())
}

#[no_mangle]
pub extern "cdecl" fn unload() -> BOOL {
  debug!("unload");

  if let Err(e) = save_global_variables() {
    error!("{}", e);
  }

  TRUE
}

#[macro_export]
macro_rules! check_error {
  ($e:expr, $err:expr) => {
    match $e {
      Ok(val) => val,
      Err(_) => return Err($err),
    }
  };
}

// unwrapがコード上に散乱するのを避けたい
#[macro_export]
macro_rules! lazy_regex {
  ($e:expr) => {
    Lazy::new(|| Regex::new($e).unwrap())
  };
}

// 同上
#[macro_export]
macro_rules! lazy_fancy_regex {
  ($e:expr) => {
    Lazy::new(|| FancyRegex::new($e).unwrap())
  };
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub extern "cdecl" fn request(h: HGLOBAL, len: &mut c_long) -> HGLOBAL {
  // リクエストの取得
  let v = GStr::capture(h, *len as usize);

  let s = if let Ok(s) = v.to_utf8_str() {
    s
  } else {
    let err = "error while decoding request";
    error!("{}", err);
    let mut res = new_response_nocontent();
    add_error_description(&mut res, err);
    let bytes = res.to_string().into_bytes();
    let response_gstr = GStr::clone_from_slice_nofree(&bytes);
    *len = response_gstr.len() as c_long;
    return response_gstr.handle();
  };

  let r = if let Ok(req) = Request::parse(s) {
    req
  } else {
    let err = format!("error while parsing request: {}", s);
    error!("{}", err);
    let mut res = new_response_nocontent();
    add_error_description(&mut res, err.as_str());
    let bytes = res.to_string().into_bytes();
    let response_gstr = GStr::clone_from_slice_nofree(&bytes);
    *len = response_gstr.len() as c_long;
    return response_gstr.handle();
  };

  let response = match events::handle_request(&r) {
    Ok(res) => res,
    Err(e) => {
      let err = format!("error while making response: {}", e);
      error!("{}", err);
      let mut res = new_response_nocontent();
      add_error_description(&mut res, err.as_str());
      res
    }
  };

  let bytes = response.to_string().into_bytes();
  let response_gstr = GStr::clone_from_slice_nofree(&bytes);
  *len = response_gstr.len() as c_long;
  response_gstr.handle()
}
