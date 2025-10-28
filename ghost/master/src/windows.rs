use winapi::um::minwinbase::SYSTEMTIME;
use winapi::um::sysinfoapi::GetLocalTime;

pub(crate) fn get_local_time() -> SYSTEMTIME {
  unsafe {
    let mut st: SYSTEMTIME = std::mem::zeroed();
    GetLocalTime(&mut st);
    st
  }
}
