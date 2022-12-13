#![windows_subsystem = "windows"]

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;

use nwd::NwgUi;
use nwg::NativeUi;
use winapi::um::winuser::{WS_EX_TRANSPARENT, WS_EX_TOOLWINDOW};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CAPITAL, VK_NUMLOCK};

const SIZE: i32 = 64;

#[derive(Default, NwgUi)]
pub struct LockIndicator {
    #[nwg_control(size: (SIZE, SIZE), flags: "POPUP", ex_flags: WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW , topmost: true)]
    window: nwg::Window,

    #[nwg_resource]
    embed: nwg::EmbedResource,

    #[nwg_resource(source_embed: Some(&data.embed), source_embed_str: Some("STATE0ICON"))]
    state0icon: nwg::Icon,
    #[nwg_resource(source_embed: Some(&data.embed), source_embed_str: Some("STATE1ICON"))]
    state1icon: nwg::Icon,
    #[nwg_resource(source_embed: Some(&data.embed), source_embed_str: Some("STATE2ICON"))]
    state2icon: nwg::Icon,
    #[nwg_resource(source_embed: Some(&data.embed), source_embed_str: Some("STATE3ICON"))]
    state3icon: nwg::Icon,

    #[nwg_control(icon: Some(&data.state0icon), tip: Some("Caps/Num Lock Indicator"))]
    #[nwg_events(MousePressLeftUp: [LockIndicator::show_menu], OnContextMenu: [LockIndicator::show_menu])]
    tray: nwg::TrayNotification,

    #[nwg_control(parent: window, popup: true)]
    tray_menu: nwg::Menu,

    #[nwg_control(parent: tray_menu, text: "Exit")]
    #[nwg_events(OnMenuItemSelected: [LockIndicator::exit])]
    tray_item: nwg::MenuItem,

    #[nwg_resource(source_file: Some("./resource/caps-lock-on.png"), size: Some((SIZE.try_into().unwrap(), SIZE.try_into().unwrap())))]
    caps_lock_on_image: nwg::Bitmap,

    #[nwg_resource(source_file: Some("./resource/caps-lock-off.png"), size: Some((SIZE.try_into().unwrap(), SIZE.try_into().unwrap())))]
    caps_lock_off_image: nwg::Bitmap,

    #[nwg_resource(source_file: Some("./resource/num-lock-on.png"), size: Some((SIZE.try_into().unwrap(), SIZE.try_into().unwrap())))]
    num_lock_on_image: nwg::Bitmap,

    #[nwg_resource(source_file: Some("./resource/num-lock-off.png"), size: Some((SIZE.try_into().unwrap(), SIZE.try_into().unwrap())))]
    num_lock_off_image: nwg::Bitmap,

    #[nwg_control(size: (SIZE, SIZE), bitmap: Some(&data.caps_lock_off_image))]
    image_frame: nwg::ImageFrame,
}

impl LockIndicator {
    fn change_icon(&self, last_state: &State, state: &State) {
        match state {
            State { caps: false, num: false } => self.tray.set_icon(&self.state0icon),
            State { caps: true, num: false } => self.tray.set_icon(&self.state1icon),
            State { caps: false, num: true } => self.tray.set_icon(&self.state2icon),
            State { caps: true, num: true } => self.tray.set_icon(&self.state3icon),
        }
        self.show_splash(last_state, state);
    }

    fn calculate_splash_position(&self) -> (i32, i32) {
        let [_left, _top, right, bottom] = nwg::Monitor::monitor_rect_from_window(&self.window);
        let (width, height) = self.window.size();
        let width: i32 = width.try_into().unwrap();
        let height: i32 = height.try_into().unwrap();

        let x = right - 24 - width;
        let y = bottom - 64 - height;
        (x, y)
    }

    fn show_splash(&self, last_state: &State, state: &State) {
        if last_state.caps == false && state.caps == true {
            self.image_frame.set_bitmap(Some(&self.caps_lock_on_image));
        } else
        if last_state.caps == true && state.caps == false {
            self.image_frame.set_bitmap(Some(&self.caps_lock_off_image));
        } else
        if last_state.num == false && state.num == true {
            self.image_frame.set_bitmap(Some(&self.num_lock_on_image));
        } else
        if last_state.num == true && state.num == false {
            self.image_frame.set_bitmap(Some(&self.num_lock_off_image));
        }

        let splash_position = self.calculate_splash_position();
        self.window.set_position(splash_position.0, splash_position.1);

        self.window.set_visible(true)
    }

    fn hide_splash(&self) {
        self.window.set_visible(false)
    }

    fn show_menu(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.tray_menu.popup(x, y)
    }
    
    fn exit(&self) {
        nwg::stop_thread_dispatch()
    }
}

struct State {
    caps: bool,
    num: bool,
}

impl State {
    fn equals(&self, comparison: &State) -> bool {
        self.caps == comparison.caps && self.num == comparison.num
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    let ui = LockIndicator::build_ui(Default::default()).expect("Failed to build UI");

    let mut last_state = State { caps: false, num: false };
    let mut timer = -1;
    nwg::dispatch_thread_events_with_callback(move || {
        let mut state = State { caps: false, num: false };

        unsafe {
            if GetKeyState(VK_CAPITAL.0.into()) == 1 { state.caps = true };
            if GetKeyState(VK_NUMLOCK.0.into()) == 1 { state.num = true };
        }

        if !last_state.equals(&state) {
            ui.change_icon(&last_state, &state);
            last_state = state;
            timer = 50;
        }

        if timer == 0 {
            ui.hide_splash();
        }

        if timer > -1 {
            timer -= 1;
        }

        std::thread::sleep(std::time::Duration::from_millis(1));
    });
}