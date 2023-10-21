


KEYS = """
      Alt,
    AltGr,
    Backspace,
    CapsLock,
    ControlLeft,
    ControlRight,
    Delete,
    DownArrow,
    End,
    Escape,
    F1,
    F10,
    F11,
    F12,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    Home,
    LeftArrow,
    MetaLeft,
    MetaRight,
    PageDown,
    PageUp,
    Return,
    RightArrow,
    ShiftLeft,
    ShiftRight,
    Space,
    Tab,
    UpArrow,
    PrintScreen,
    ScrollLock,
    Pause,
    NumLock,
    BackQuote,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Num0,
    Minus,
    Equal,
    KeyQ,
    KeyW,
    KeyE,
    KeyR,
    KeyT,
    KeyY,
    KeyU,
    KeyI,
    KeyO,
    KeyP,
    LeftBracket,
    RightBracket,
    KeyA,
    KeyS,
    KeyD,
    KeyF,
    KeyG,
    KeyH,
    KeyJ,
    KeyK,
    KeyL,
    SemiColon,
    Quote,
    BackSlash,
    IntlBackslash,
    KeyZ,
    KeyX,
    KeyC,
    KeyV,
    KeyB,
    KeyN,
    KeyM,
    Comma,
    Dot,
    Slash,
    Insert,
    KpReturn,
    KpMinus,
    KpPlus,
    KpMultiply,
    KpDivide,
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpDelete,
    Function,
    """


def print_proto():
  print("enum KeyCode {")
  print("\tUNKNOWN_KEY = 0;")
  index = 1
  for key in KEYS.split(','):
    key = key.strip()
    if not key:
      continue
    print(f"\t{key.upper()} = {index};")
    index += 1
  print("}")



def print_match():
  for key in KEYS.split(','):
    key = key.strip()
    if not key:
      continue
    rkey = key
    rkey = rkey.lower()
    rkey = rkey[:1].upper() + rkey[1:]
    print(f"              msg::KeyCode::{rkey} => Some(rdev::Key::{key}),")
    
    
def print_backward():
  for key in KEYS.split(','):
    key = key.strip()
    if not key:
      continue
    rkey = key
    rkey = rkey.lower()
    rkey = rkey[:1].upper() + rkey[1:]
    
    print(f"        rdev::Key::{key} => msg::key::Key::Code(msg::KeyCode::{rkey} as i32),")



print(10 * '\n')
print_backward()