use druid::widget::{Button, Flex, Label};
use druid::AppLauncher;
use druid::LocalizedString;
use druid::Widget;
use druid::WidgetExt;
use druid::WindowDesc;

use crate::events::ControlEvent;
use std::sync::mpsc::Sender;

fn ui_builder() -> impl Widget<u32> {
  // The label text will be computed dynamically based on the current locale and count
  let text = LocalizedString::new("hello-counter")
    .with_arg("count", |data: &u32, _env| (*data).into());
  let label = Label::new(text).padding(5.0).center();
  let button = Button::new("increment")
    .on_click(|_ctx, data, _env| *data += 1)
    .padding(5.0);
  Flex::column().with_child(label).with_child(button)
}

pub fn launch_display(sender: Sender<ControlEvent>) {
  let main_window = WindowDesc::new(ui_builder());
  let data = 0_u32;
  let _ = AppLauncher::with_window(main_window)
    .log_to_console()
    .launch(data);

  sender.send(ControlEvent::StartListening).unwrap();
}
