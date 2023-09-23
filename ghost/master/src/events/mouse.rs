use crate::variables::GlobalVariables;
use std::collections::HashMap;

pub type MouseDialogs = HashMap<String, Vec<String>>;

pub fn wheel_dialogs(_vars: &mut GlobalVariables) -> MouseDialogs {
    let mut dialogs: MouseDialogs = HashMap::new();

    dialogs.insert("0skirtup".to_string(), vec!["スカートめくり".to_string()]);
    dialogs.insert("0shoulderdown".to_string(), vec!["抱き寄せる".to_string()]);

    dialogs
}
