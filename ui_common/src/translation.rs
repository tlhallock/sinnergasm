use rdev;
use sinnergasm::protos as msg;

pub fn mouse_msg_to_rdev(button: &msg::MouseButton) -> Option<rdev::Button> {
  if let Some(button) = &button.r#type {
    match button {
        msg::mouse_button::Type::Button(button) => {
          if let Some(button) = msg::MouseButtons::from_i32(*button) {
            match button {
              msg::MouseButtons::Left => Some(rdev::Button::Left),
              msg::MouseButtons::Middle => Some(rdev::Button::Middle),
              msg::MouseButtons::Right => Some(rdev::Button::Right),
              msg::MouseButtons::UnknownButton => None,
            }
          } else {
            None
          }
        }
        msg::mouse_button::Type::Other(code) => u8::try_from(*code).ok().map(rdev::Button::Unknown),
    }
  } else {
    None
  }
}

pub fn mouse_rdev_to_msg(button: rdev::Button) -> msg::MouseButton {
  msg::MouseButton {
    r#type: Some(match button {
      rdev::Button::Left => msg::mouse_button::Type::Button(msg::MouseButtons::Left as i32),
      rdev::Button::Right => msg::mouse_button::Type::Button(msg::MouseButtons::Right as i32),
      rdev::Button::Middle => msg::mouse_button::Type::Button(msg::MouseButtons::Middle as i32),
      rdev::Button::Unknown(code) => msg::mouse_button::Type::Other(code.into()),
    })
  }
}


pub fn rdev_to_msg(key: &rdev::Key) -> msg::Key {
  msg::Key {
    key: Some(
      match key {
        rdev::Key::Alt => msg::key::Key::Code(msg::KeyCode::Alt as i32),
        rdev::Key::AltGr => msg::key::Key::Code(msg::KeyCode::Altgr as i32),
        rdev::Key::Backspace => msg::key::Key::Code(msg::KeyCode::Backspace as i32),
        rdev::Key::CapsLock => msg::key::Key::Code(msg::KeyCode::Capslock as i32),
        rdev::Key::ControlLeft => msg::key::Key::Code(msg::KeyCode::Controlleft as i32),
        rdev::Key::ControlRight => msg::key::Key::Code(msg::KeyCode::Controlright as i32),
        rdev::Key::Delete => msg::key::Key::Code(msg::KeyCode::Delete as i32),
        rdev::Key::DownArrow => msg::key::Key::Code(msg::KeyCode::Downarrow as i32),
        rdev::Key::End => msg::key::Key::Code(msg::KeyCode::End as i32),
        rdev::Key::Escape => msg::key::Key::Code(msg::KeyCode::Escape as i32),
        rdev::Key::F1 => msg::key::Key::Code(msg::KeyCode::F1 as i32),
        rdev::Key::F10 => msg::key::Key::Code(msg::KeyCode::F10 as i32),
        rdev::Key::F11 => msg::key::Key::Code(msg::KeyCode::F11 as i32),
        rdev::Key::F12 => msg::key::Key::Code(msg::KeyCode::F12 as i32),
        rdev::Key::F2 => msg::key::Key::Code(msg::KeyCode::F2 as i32),
        rdev::Key::F3 => msg::key::Key::Code(msg::KeyCode::F3 as i32),
        rdev::Key::F4 => msg::key::Key::Code(msg::KeyCode::F4 as i32),
        rdev::Key::F5 => msg::key::Key::Code(msg::KeyCode::F5 as i32),
        rdev::Key::F6 => msg::key::Key::Code(msg::KeyCode::F6 as i32),
        rdev::Key::F7 => msg::key::Key::Code(msg::KeyCode::F7 as i32),
        rdev::Key::F8 => msg::key::Key::Code(msg::KeyCode::F8 as i32),
        rdev::Key::F9 => msg::key::Key::Code(msg::KeyCode::F9 as i32),
        rdev::Key::Home => msg::key::Key::Code(msg::KeyCode::Home as i32),
        rdev::Key::LeftArrow => msg::key::Key::Code(msg::KeyCode::Leftarrow as i32),
        rdev::Key::MetaLeft => msg::key::Key::Code(msg::KeyCode::Metaleft as i32),
        rdev::Key::MetaRight => msg::key::Key::Code(msg::KeyCode::Metaright as i32),
        rdev::Key::PageDown => msg::key::Key::Code(msg::KeyCode::Pagedown as i32),
        rdev::Key::PageUp => msg::key::Key::Code(msg::KeyCode::Pageup as i32),
        rdev::Key::Return => msg::key::Key::Code(msg::KeyCode::Return as i32),
        rdev::Key::RightArrow => msg::key::Key::Code(msg::KeyCode::Rightarrow as i32),
        rdev::Key::ShiftLeft => msg::key::Key::Code(msg::KeyCode::Shiftleft as i32),
        rdev::Key::ShiftRight => msg::key::Key::Code(msg::KeyCode::Shiftright as i32),
        rdev::Key::Space => msg::key::Key::Code(msg::KeyCode::Space as i32),
        rdev::Key::Tab => msg::key::Key::Code(msg::KeyCode::Tab as i32),
        rdev::Key::UpArrow => msg::key::Key::Code(msg::KeyCode::Uparrow as i32),
        rdev::Key::PrintScreen => msg::key::Key::Code(msg::KeyCode::Printscreen as i32),
        rdev::Key::ScrollLock => msg::key::Key::Code(msg::KeyCode::Scrolllock as i32),
        rdev::Key::Pause => msg::key::Key::Code(msg::KeyCode::Pause as i32),
        rdev::Key::NumLock => msg::key::Key::Code(msg::KeyCode::Numlock as i32),
        rdev::Key::BackQuote => msg::key::Key::Code(msg::KeyCode::Backquote as i32),
        rdev::Key::Num1 => msg::key::Key::Code(msg::KeyCode::Num1 as i32),
        rdev::Key::Num2 => msg::key::Key::Code(msg::KeyCode::Num2 as i32),
        rdev::Key::Num3 => msg::key::Key::Code(msg::KeyCode::Num3 as i32),
        rdev::Key::Num4 => msg::key::Key::Code(msg::KeyCode::Num4 as i32),
        rdev::Key::Num5 => msg::key::Key::Code(msg::KeyCode::Num5 as i32),
        rdev::Key::Num6 => msg::key::Key::Code(msg::KeyCode::Num6 as i32),
        rdev::Key::Num7 => msg::key::Key::Code(msg::KeyCode::Num7 as i32),
        rdev::Key::Num8 => msg::key::Key::Code(msg::KeyCode::Num8 as i32),
        rdev::Key::Num9 => msg::key::Key::Code(msg::KeyCode::Num9 as i32),
        rdev::Key::Num0 => msg::key::Key::Code(msg::KeyCode::Num0 as i32),
        rdev::Key::Minus => msg::key::Key::Code(msg::KeyCode::Minus as i32),
        rdev::Key::Equal => msg::key::Key::Code(msg::KeyCode::Equal as i32),
        rdev::Key::KeyQ => msg::key::Key::Code(msg::KeyCode::Keyq as i32),
        rdev::Key::KeyW => msg::key::Key::Code(msg::KeyCode::Keyw as i32),
        rdev::Key::KeyE => msg::key::Key::Code(msg::KeyCode::Keye as i32),
        rdev::Key::KeyR => msg::key::Key::Code(msg::KeyCode::Keyr as i32),
        rdev::Key::KeyT => msg::key::Key::Code(msg::KeyCode::Keyt as i32),
        rdev::Key::KeyY => msg::key::Key::Code(msg::KeyCode::Keyy as i32),
        rdev::Key::KeyU => msg::key::Key::Code(msg::KeyCode::Keyu as i32),
        rdev::Key::KeyI => msg::key::Key::Code(msg::KeyCode::Keyi as i32),
        rdev::Key::KeyO => msg::key::Key::Code(msg::KeyCode::Keyo as i32),
        rdev::Key::KeyP => msg::key::Key::Code(msg::KeyCode::Keyp as i32),
        rdev::Key::LeftBracket => msg::key::Key::Code(msg::KeyCode::Leftbracket as i32),
        rdev::Key::RightBracket => msg::key::Key::Code(msg::KeyCode::Rightbracket as i32),
        rdev::Key::KeyA => msg::key::Key::Code(msg::KeyCode::Keya as i32),
        rdev::Key::KeyS => msg::key::Key::Code(msg::KeyCode::Keys as i32),
        rdev::Key::KeyD => msg::key::Key::Code(msg::KeyCode::Keyd as i32),
        rdev::Key::KeyF => msg::key::Key::Code(msg::KeyCode::Keyf as i32),
        rdev::Key::KeyG => msg::key::Key::Code(msg::KeyCode::Keyg as i32),
        rdev::Key::KeyH => msg::key::Key::Code(msg::KeyCode::Keyh as i32),
        rdev::Key::KeyJ => msg::key::Key::Code(msg::KeyCode::Keyj as i32),
        rdev::Key::KeyK => msg::key::Key::Code(msg::KeyCode::Keyk as i32),
        rdev::Key::KeyL => msg::key::Key::Code(msg::KeyCode::Keyl as i32),
        rdev::Key::SemiColon => msg::key::Key::Code(msg::KeyCode::Semicolon as i32),
        rdev::Key::Quote => msg::key::Key::Code(msg::KeyCode::Quote as i32),
        rdev::Key::BackSlash => msg::key::Key::Code(msg::KeyCode::Backslash as i32),
        rdev::Key::IntlBackslash => msg::key::Key::Code(msg::KeyCode::Intlbackslash as i32),
        rdev::Key::KeyZ => msg::key::Key::Code(msg::KeyCode::Keyz as i32),
        rdev::Key::KeyX => msg::key::Key::Code(msg::KeyCode::Keyx as i32),
        rdev::Key::KeyC => msg::key::Key::Code(msg::KeyCode::Keyc as i32),
        rdev::Key::KeyV => msg::key::Key::Code(msg::KeyCode::Keyv as i32),
        rdev::Key::KeyB => msg::key::Key::Code(msg::KeyCode::Keyb as i32),
        rdev::Key::KeyN => msg::key::Key::Code(msg::KeyCode::Keyn as i32),
        rdev::Key::KeyM => msg::key::Key::Code(msg::KeyCode::Keym as i32),
        rdev::Key::Comma => msg::key::Key::Code(msg::KeyCode::Comma as i32),
        rdev::Key::Dot => msg::key::Key::Code(msg::KeyCode::Dot as i32),
        rdev::Key::Slash => msg::key::Key::Code(msg::KeyCode::Slash as i32),
        rdev::Key::Insert => msg::key::Key::Code(msg::KeyCode::Insert as i32),
        rdev::Key::KpReturn => msg::key::Key::Code(msg::KeyCode::Kpreturn as i32),
        rdev::Key::KpMinus => msg::key::Key::Code(msg::KeyCode::Kpminus as i32),
        rdev::Key::KpPlus => msg::key::Key::Code(msg::KeyCode::Kpplus as i32),
        rdev::Key::KpMultiply => msg::key::Key::Code(msg::KeyCode::Kpmultiply as i32),
        rdev::Key::KpDivide => msg::key::Key::Code(msg::KeyCode::Kpdivide as i32),
        rdev::Key::Kp0 => msg::key::Key::Code(msg::KeyCode::Kp0 as i32),
        rdev::Key::Kp1 => msg::key::Key::Code(msg::KeyCode::Kp1 as i32),
        rdev::Key::Kp2 => msg::key::Key::Code(msg::KeyCode::Kp2 as i32),
        rdev::Key::Kp3 => msg::key::Key::Code(msg::KeyCode::Kp3 as i32),
        rdev::Key::Kp4 => msg::key::Key::Code(msg::KeyCode::Kp4 as i32),
        rdev::Key::Kp5 => msg::key::Key::Code(msg::KeyCode::Kp5 as i32),
        rdev::Key::Kp6 => msg::key::Key::Code(msg::KeyCode::Kp6 as i32),
        rdev::Key::Kp7 => msg::key::Key::Code(msg::KeyCode::Kp7 as i32),
        rdev::Key::Kp8 => msg::key::Key::Code(msg::KeyCode::Kp8 as i32),
        rdev::Key::Kp9 => msg::key::Key::Code(msg::KeyCode::Kp9 as i32),
        rdev::Key::KpDelete => msg::key::Key::Code(msg::KeyCode::Kpdelete as i32),
        rdev::Key::Function => msg::key::Key::Code(msg::KeyCode::Function as i32),
        rdev::Key::Unknown(key_code) => msg::key::Key::Other(*key_code)
      }
    )
  }
}

pub fn msg_to_rdev(key: &msg::Key) -> Option<rdev::Key> {
  if let Some(key) = &key.key {
    match key {
      msg::key::Key::Code(key_code) => {
        let kc = msg::KeyCode::from_i32(*key_code);
        if let Some(kc) = kc {
          match kc {
            msg::KeyCode::UnknownKey => None,
            msg::KeyCode::Alt => Some(rdev::Key::Alt),
            msg::KeyCode::Altgr => Some(rdev::Key::AltGr),
            msg::KeyCode::Backspace => Some(rdev::Key::Backspace),
            msg::KeyCode::Capslock => Some(rdev::Key::CapsLock),
            msg::KeyCode::Controlleft => Some(rdev::Key::ControlLeft),
            msg::KeyCode::Controlright => Some(rdev::Key::ControlRight),
            msg::KeyCode::Delete => Some(rdev::Key::Delete),
            msg::KeyCode::Downarrow => Some(rdev::Key::DownArrow),
            msg::KeyCode::End => Some(rdev::Key::End),
            msg::KeyCode::Escape => Some(rdev::Key::Escape),
            msg::KeyCode::F1 => Some(rdev::Key::F1),
            msg::KeyCode::F10 => Some(rdev::Key::F10),
            msg::KeyCode::F11 => Some(rdev::Key::F11),
            msg::KeyCode::F12 => Some(rdev::Key::F12),
            msg::KeyCode::F2 => Some(rdev::Key::F2),
            msg::KeyCode::F3 => Some(rdev::Key::F3),
            msg::KeyCode::F4 => Some(rdev::Key::F4),
            msg::KeyCode::F5 => Some(rdev::Key::F5),
            msg::KeyCode::F6 => Some(rdev::Key::F6),
            msg::KeyCode::F7 => Some(rdev::Key::F7),
            msg::KeyCode::F8 => Some(rdev::Key::F8),
            msg::KeyCode::F9 => Some(rdev::Key::F9),
            msg::KeyCode::Home => Some(rdev::Key::Home),
            msg::KeyCode::Leftarrow => Some(rdev::Key::LeftArrow),
            msg::KeyCode::Metaleft => Some(rdev::Key::MetaLeft),
            msg::KeyCode::Metaright => Some(rdev::Key::MetaRight),
            msg::KeyCode::Pagedown => Some(rdev::Key::PageDown),
            msg::KeyCode::Pageup => Some(rdev::Key::PageUp),
            msg::KeyCode::Return => Some(rdev::Key::Return),
            msg::KeyCode::Rightarrow => Some(rdev::Key::RightArrow),
            msg::KeyCode::Shiftleft => Some(rdev::Key::ShiftLeft),
            msg::KeyCode::Shiftright => Some(rdev::Key::ShiftRight),
            msg::KeyCode::Space => Some(rdev::Key::Space),
            msg::KeyCode::Tab => Some(rdev::Key::Tab),
            msg::KeyCode::Uparrow => Some(rdev::Key::UpArrow),
            msg::KeyCode::Printscreen => Some(rdev::Key::PrintScreen),
            msg::KeyCode::Scrolllock => Some(rdev::Key::ScrollLock),
            msg::KeyCode::Pause => Some(rdev::Key::Pause),
            msg::KeyCode::Numlock => Some(rdev::Key::NumLock),
            msg::KeyCode::Backquote => Some(rdev::Key::BackQuote),
            msg::KeyCode::Num1 => Some(rdev::Key::Num1),
            msg::KeyCode::Num2 => Some(rdev::Key::Num2),
            msg::KeyCode::Num3 => Some(rdev::Key::Num3),
            msg::KeyCode::Num4 => Some(rdev::Key::Num4),
            msg::KeyCode::Num5 => Some(rdev::Key::Num5),
            msg::KeyCode::Num6 => Some(rdev::Key::Num6),
            msg::KeyCode::Num7 => Some(rdev::Key::Num7),
            msg::KeyCode::Num8 => Some(rdev::Key::Num8),
            msg::KeyCode::Num9 => Some(rdev::Key::Num9),
            msg::KeyCode::Num0 => Some(rdev::Key::Num0),
            msg::KeyCode::Minus => Some(rdev::Key::Minus),
            msg::KeyCode::Equal => Some(rdev::Key::Equal),
            msg::KeyCode::Keyq => Some(rdev::Key::KeyQ),
            msg::KeyCode::Keyw => Some(rdev::Key::KeyW),
            msg::KeyCode::Keye => Some(rdev::Key::KeyE),
            msg::KeyCode::Keyr => Some(rdev::Key::KeyR),
            msg::KeyCode::Keyt => Some(rdev::Key::KeyT),
            msg::KeyCode::Keyy => Some(rdev::Key::KeyY),
            msg::KeyCode::Keyu => Some(rdev::Key::KeyU),
            msg::KeyCode::Keyi => Some(rdev::Key::KeyI),
            msg::KeyCode::Keyo => Some(rdev::Key::KeyO),
            msg::KeyCode::Keyp => Some(rdev::Key::KeyP),
            msg::KeyCode::Leftbracket => Some(rdev::Key::LeftBracket),
            msg::KeyCode::Rightbracket => Some(rdev::Key::RightBracket),
            msg::KeyCode::Keya => Some(rdev::Key::KeyA),
            msg::KeyCode::Keys => Some(rdev::Key::KeyS),
            msg::KeyCode::Keyd => Some(rdev::Key::KeyD),
            msg::KeyCode::Keyf => Some(rdev::Key::KeyF),
            msg::KeyCode::Keyg => Some(rdev::Key::KeyG),
            msg::KeyCode::Keyh => Some(rdev::Key::KeyH),
            msg::KeyCode::Keyj => Some(rdev::Key::KeyJ),
            msg::KeyCode::Keyk => Some(rdev::Key::KeyK),
            msg::KeyCode::Keyl => Some(rdev::Key::KeyL),
            msg::KeyCode::Semicolon => Some(rdev::Key::SemiColon),
            msg::KeyCode::Quote => Some(rdev::Key::Quote),
            msg::KeyCode::Backslash => Some(rdev::Key::BackSlash),
            msg::KeyCode::Intlbackslash => Some(rdev::Key::IntlBackslash),
            msg::KeyCode::Keyz => Some(rdev::Key::KeyZ),
            msg::KeyCode::Keyx => Some(rdev::Key::KeyX),
            msg::KeyCode::Keyc => Some(rdev::Key::KeyC),
            msg::KeyCode::Keyv => Some(rdev::Key::KeyV),
            msg::KeyCode::Keyb => Some(rdev::Key::KeyB),
            msg::KeyCode::Keyn => Some(rdev::Key::KeyN),
            msg::KeyCode::Keym => Some(rdev::Key::KeyM),
            msg::KeyCode::Comma => Some(rdev::Key::Comma),
            msg::KeyCode::Dot => Some(rdev::Key::Dot),
            msg::KeyCode::Slash => Some(rdev::Key::Slash),
            msg::KeyCode::Insert => Some(rdev::Key::Insert),
            msg::KeyCode::Kpreturn => Some(rdev::Key::KpReturn),
            msg::KeyCode::Kpminus => Some(rdev::Key::KpMinus),
            msg::KeyCode::Kpplus => Some(rdev::Key::KpPlus),
            msg::KeyCode::Kpmultiply => Some(rdev::Key::KpMultiply),
            msg::KeyCode::Kpdivide => Some(rdev::Key::KpDivide),
            msg::KeyCode::Kp0 => Some(rdev::Key::Kp0),
            msg::KeyCode::Kp1 => Some(rdev::Key::Kp1),
            msg::KeyCode::Kp2 => Some(rdev::Key::Kp2),
            msg::KeyCode::Kp3 => Some(rdev::Key::Kp3),
            msg::KeyCode::Kp4 => Some(rdev::Key::Kp4),
            msg::KeyCode::Kp5 => Some(rdev::Key::Kp5),
            msg::KeyCode::Kp6 => Some(rdev::Key::Kp6),
            msg::KeyCode::Kp7 => Some(rdev::Key::Kp7),
            msg::KeyCode::Kp8 => Some(rdev::Key::Kp8),
            msg::KeyCode::Kp9 => Some(rdev::Key::Kp9),
            msg::KeyCode::Kpdelete => Some(rdev::Key::KpDelete),
            msg::KeyCode::Function => Some(rdev::Key::Function),
          }
        } else {
          None
        }
      }
      msg::key::Key::Other(key_code) => Some(rdev::Key::Unknown(*key_code)),
    }
  } else {
    None
  }
}


