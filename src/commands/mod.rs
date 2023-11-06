use serenity::framework::StandardFramework;

mod femboy;
mod shipu;

use crate::service::user::UserError;

use self::{femboy::FEMBOY_GROUP, shipu::SHIPU_GROUP};

fn handle_user_error(e: UserError) -> String {
    match e {
        UserError::AlreadyExists => t!("msg.user.common.error.already_exists"),
    }
}

pub fn register_commands(framework: StandardFramework) -> StandardFramework {
    framework.group(&SHIPU_GROUP).group(&FEMBOY_GROUP)
}
