use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};

static mut PLAYER: Option<Player> = None;

pub(crate) struct Player {
  // 直接アクセスされることはない
  // ただし、drop時にストリームが閉じられるため、変数として保持しておく必要がある
  stream: OutputStream,
  stream_handle: OutputStreamHandle,
  handler: Option<std::thread::JoinHandle<()>>,
  stop_flag: Arc<Mutex<bool>>,
  sinks: Vec<Sink>,
}

impl Player {
  pub fn new() -> Option<Self> {
    let (stream, stream_handle) = match OutputStream::try_default() {
      Ok(v) => v,
      Err(_) => return None,
    };
    Some(Player {
      stream,
      stream_handle,
      handler: Some(std::thread::spawn(move || loop {
        let player = get_player();
        if let Some(p) = player {
          if *p.stop_flag.lock().unwrap() {
            break;
          }
          p.sinks.retain(|sink| {
            if sink.empty() {
              sink.stop();
              false
            } else {
              true
            }
          });
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
      })),
      stop_flag: Arc::new(Mutex::new(false)),
      sinks: Vec::new(),
    })
  }

  fn reset_device(&mut self) {
    let (stream, stream_handle) = OutputStream::try_default().unwrap();
    self.stream = stream;
    self.stream_handle = stream_handle;
    self.sinks.clear();
  }
}

pub(crate) fn force_free_player() {
  debug!("free_player");
  let player = get_player();
  if let Some(p) = player {
    *p.stop_flag.lock().unwrap() = true;
    p.handler.take().unwrap().join().unwrap();
    while let Some(sink) = p.sinks.pop() {
      sink.pause();
      sink.stop();
    }
    unsafe {
      PLAYER = None;
    }
  }
  debug!("force_free_player done");
}

pub(crate) fn cooperative_free_player() {
  debug!("sleep until end");
  let player = get_player();
  if let Some(p) = player {
    *p.stop_flag.lock().unwrap() = true;
    p.handler.take().unwrap().join().unwrap();
    while let Some(sink) = p.sinks.pop() {
      while !sink.empty() {
        std::thread::sleep(std::time::Duration::from_millis(100));
      }
      sink.pause();
      sink.stop();
    }
    unsafe {
      PLAYER = None;
    }
  }
  debug!("cooperative_free_player done");
}

pub(crate) fn get_player() -> Option<&'static mut Player> {
  if unsafe { PLAYER.is_none() } {
    unsafe {
      PLAYER = Player::new();
    }
  }
  unsafe { PLAYER.as_mut() }
}

pub(crate) fn play_sound(file: &str) -> Result<(), Box<dyn std::error::Error>> {
  let player = if let Some(p) = get_player() {
    p
  } else {
    // プレイヤーが存在しない場合は何もしない
    return Ok(());
  };
  if player.sinks.len() >= 10 {
    // 再生する前に、一度デバイスをリセットする
    // 再生デバイスが変更されていた場合に対応するため
    player.reset_device();
  }
  let reader = BufReader::new(File::open(file)?);
  let sink = player.stream_handle.play_once(reader)?;
  player.sinks.push(sink);
  Ok(())
}
