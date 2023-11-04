use serenity::framework::StandardFramework;

mod femboy;
mod shipu;

use self::{femboy::FEMBOY_GROUP, shipu::SHIPU_GROUP};

pub fn register_commands(framework: StandardFramework) -> StandardFramework {
    framework.group(&SHIPU_GROUP).group(&FEMBOY_GROUP)
}
