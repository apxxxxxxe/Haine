use shiorust::message::{traits::*, Request};

#[derive(Clone, Debug)]
pub struct Status {
  pub talking: bool,
  pub choosing: bool,
  pub minimizing: bool,
  pub induction: bool,
  pub passive: bool,
  pub timecritical: bool,
  pub nouserbreak: bool,
  pub online: bool,
  // opening
  // balloon
}

impl Status {
  pub fn from_str(status: &str) -> Self {
    Self {
      talking: status.contains("talking"),
      choosing: status.contains("choosing"),
      minimizing: status.contains("minimizing"),
      induction: status.contains("induction"),
      passive: status.contains("passive"),
      timecritical: status.contains("timecritical"),
      nouserbreak: status.contains("nouserbreak"),
      online: status.contains("online"),
    }
  }

  pub fn from_request(req: &Request) -> Option<Self> {
    if let Some(status) = req.headers.get("Status") {
      return Some(Self::from_str(status));
    }
    None
  }
}
