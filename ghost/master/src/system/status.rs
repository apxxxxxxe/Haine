use shiorust::message::{traits::*, Request};
use std::fmt::Display;

#[derive(Clone, Debug, Default)]
pub(crate) struct Status {
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

impl Display for Status {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut status = Vec::new();
    if self.talking {
      status.push("talking");
    }
    if self.choosing {
      status.push("choosing");
    }
    if self.minimizing {
      status.push("minimizing");
    }
    if self.induction {
      status.push("induction");
    }
    if self.passive {
      status.push("passive");
    }
    if self.timecritical {
      status.push("timecritical");
    }
    if self.nouserbreak {
      status.push("nouserbreak");
    }
    if self.online {
      status.push("online");
    }
    write!(f, "{}", status.join(","))
  }
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

  pub fn from_request(req: &Request) -> Self {
    if let Some(status) = req.headers.get("Status") {
      Self::from_str(status)
    } else {
      Self::default()
    }
  }
}
