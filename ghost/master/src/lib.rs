mod autobreakline;
mod events;
mod roulette;
mod status;
mod variables;

use crate::variables::get_global_vars;

use std::fs::File;
use std::panic;
use std::path::Path;

use shiori_hglobal::*;
use shiorust::message::*;

use winapi::ctypes::c_long;
use winapi::shared::minwindef::{BOOL, HGLOBAL, TRUE};

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
  let s = v.to_utf8_str().unwrap();

  let vars = get_global_vars();

  if let Err(e) = vars.load() {
    error!("{}", e);
  }

  // ログの設定
  let log_path = Path::new(&s.to_string()).join("haine.log");
  vars
    .volatility
    .set_log_path(log_path.to_str().unwrap().to_string());
  WriteLogger::init(
    LevelFilter::Debug,
    Config::default(),
    File::create(log_path).unwrap(),
  )
  .unwrap();

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

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "cdecl" fn request(h: HGLOBAL, len: *mut c_long) -> HGLOBAL {
  // リクエストの取得
  let v = unsafe { GStr::capture(h, *len as usize) };

  let s = v.to_utf8_str().unwrap();

  let r = Request::parse(s).unwrap();

  let response = events::handle_request(&r);

  let bytes = response.to_string().into_bytes();
  let response_gstr = GStr::clone_from_slice_nofree(&bytes);

  unsafe { *len = response_gstr.len() as c_long };
  response_gstr.handle()
}
