#[macro_use]
mod events;

mod autobreakline;
mod error;
mod roulette;
mod status;
mod variables;

use crate::events::common::{add_error_description, new_response_nocontent};
use crate::variables::get_global_vars;

use std::fs::File;
use std::panic;
use std::path::Path;

use shiori_hglobal::*;
use shiorust::message::*;

use winapi::ctypes::c_long;
use winapi::shared::minwindef::{BOOL, FALSE, HGLOBAL, TRUE};

#[macro_use]
extern crate log;
extern crate simplelog;

use simplelog::*;

#[derive(Debug)]
pub enum ResponseError {
  DecodeFailed,
}

#[no_mangle]
pub extern "cdecl" fn load(h: HGLOBAL, len: c_long) -> BOOL {
  let v = GStr::capture(h, len as usize);
  let s = if let Ok(s) = v.to_utf8_str() {
    s
  } else {
    error!("error while decoding path");
    return FALSE; // SSPの仕様上意味なし
  };

  let vars = get_global_vars();

  if let Err(e) = vars.load() {
    error!("{}", e);
  }

  // ログの設定
  let log_path = Path::new(&s.to_string()).join("haine.log");
  vars
    .volatility
    .set_log_path(log_path.to_str().unwrap().to_string());
  let fp = if let Ok(fp) = File::create(log_path) {
    fp
  } else {
    error!("error while creating log file");
    return FALSE; // SSPの仕様上意味なし
  };
  match WriteLogger::init(LevelFilter::Debug, Config::default(), fp) {
    Ok(_) => {}
    Err(e) => {
      error!("{}", e);
      return FALSE; // SSPの仕様上意味なし
    }
  }

  panic::set_hook(Box::new(|panic_info| {
    debug!("{}", panic_info);
  }));

  debug!("load");

  TRUE
}

#[no_mangle]
pub extern "cdecl" fn unload() -> BOOL {
  debug!("unload");

  if let Err(e) = get_global_vars().save() {
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
pub unsafe extern "cdecl" fn request(h: HGLOBAL, len: *mut c_long) -> HGLOBAL {
  // リクエストの取得
  let v = unsafe { GStr::capture(h, *len as usize) };

  let s = if let Ok(s) = v.to_utf8_str() {
    s
  } else {
    let err = "error while decoding request";
    error!("{}", err);
    let mut res = new_response_nocontent();
    add_error_description(&mut res, err);
    let bytes = res.to_string().into_bytes();
    let response_gstr = GStr::clone_from_slice_nofree(&bytes);
    unsafe { *len = response_gstr.len() as c_long };
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
    unsafe { *len = response_gstr.len() as c_long };
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
  unsafe { *len = response_gstr.len() as c_long };
  response_gstr.handle()
}
