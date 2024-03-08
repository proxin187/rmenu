use x11::xlib;

use std::ffi::CStr;
use std::ptr;
use std::mem;


#[derive(Clone, Copy)]
pub struct Color {
    r: u64,
    g: u64,
    b: u64,
}

impl Color {
    pub fn new(r: u64, g: u64, b: u64) -> Color {
        Color {
            r,
            g,
            b,
        }
    }

    pub fn from_str(rgb: &str) -> Result<Color, Box<dyn std::error::Error>> {
        if !rgb.is_empty() {
            let rgb = rgb.split('-').collect::<Vec<&str>>();

            if rgb.len() == 3 {
                Ok(Color::new(rgb[0].parse()?, rgb[1].parse()?, rgb[2].parse()?))
            } else {
                Err("wrong rgb formatting".into())
            }
        } else {
            Ok(Color::new(0, 0, 0))
        }
    }

    pub fn encode(&self) -> u64 {
        self.b + (self.g << 8) + (self.r << 16)
    }
}

pub struct Display {
    dpy: *mut xlib::_XDisplay,
    gc: *mut xlib::_XGC,
    window: u64,

    pub width: i32,
}

impl Drop for Display {
    fn drop(&mut self) {
        unsafe {
            xlib::XFreeGC(self.dpy, self.gc);
            xlib::XDestroyWindow(self.dpy, self.window);
            xlib::XCloseDisplay(self.dpy);
        }
    }
}

impl Display {
    pub fn open() -> Result<Display, Box<dyn std::error::Error>> {
        let dpy = unsafe { xlib::XOpenDisplay(ptr::null()) };

        if dpy.is_null() {
            Err("failed to open display".into())
        } else {
            unsafe {
                let mut attr: xlib::XWindowAttributes = mem::zeroed();
                let mut values: xlib::XGCValues = mem::zeroed();
                let mut focused: u64 = 0;
                let mut revert_to: i32 = 0;
                let mut current_monitor = x11::xinerama::XineramaScreenInfo {
                    screen_number: 0,
                    x_org: 0,
                    y_org: 0,
                    width: 0,
                    height: 0,
                };

                xlib::XGetInputFocus(dpy, &mut focused, &mut revert_to);
                xlib::XGetWindowAttributes(dpy, focused, &mut attr);

                let monitors = x11::xinerama::XineramaQueryScreens(dpy, &mut 2);

                let mut root_x = attr.x;

                // if we are root window, get mouse position instead of window position
                if focused == xlib::XDefaultRootWindow(dpy) {
                    let mut root_return = xlib::XDefaultRootWindow(dpy);

                    xlib::XQueryPointer(
                        dpy,
                        root_return,
                        &mut root_return,
                        &mut root_return,
                        &mut root_x,
                        &mut 0,
                        &mut 0,
                        &mut 0,
                        &mut 0,
                    );
                }

                for i in 0..2 {
                    let monitor = *monitors.offset(i);

                    let range = monitor.x_org as i32..monitor.width as i32 + monitor.x_org as i32;

                    if range.contains(&root_x) {
                        current_monitor = monitor;
                    }
                }

                let black = Color::new(40, 40, 40).encode();
                let window = xlib::XCreateSimpleWindow(
                    dpy,
                    xlib::XDefaultRootWindow(dpy),
                    current_monitor.x_org as i32,
                    current_monitor.y_org as i32,
                    current_monitor.width as u32,
                    22,
                    0,
                    black,
                    black
                );

                let gc = xlib::XCreateGC(dpy, window, 0, &mut values);

                xlib::XSync(dpy, xlib::False);

                Ok(Display {
                    dpy,
                    gc,
                    window,

                    width: current_monitor.width as i32,
                })
            }
        }
    }

    pub fn clear_window(&mut self) {
        unsafe {
            xlib::XClearWindow(self.dpy, self.window);
        }
    }

    pub fn map_window(&mut self) {
        unsafe {
            xlib::XMapWindow(self.dpy, self.window);
        }
    }

    fn null_terminate(&self, string: &str) -> String {
        format!("{}\0", string)
    }

    pub fn set_property(&mut self, property: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let p_atom = xlib::XInternAtom(self.dpy, self.null_terminate(property).as_ptr() as *const i8, xlib::False);
            let v_atom = xlib::XInternAtom(self.dpy, self.null_terminate(value).as_ptr() as *const i8, xlib::False);

            xlib::XChangeProperty(self.dpy, self.window, p_atom, xlib::XA_ATOM, 32, xlib::PropModeReplace, (&v_atom as *const u64) as *const u8, 1);
        }

        Ok(())
    }

    pub fn poll_event(&mut self) -> Option<xlib::XEvent> {
        unsafe {
            if xlib::XPending(self.dpy) > 0 {
                let mut event: xlib::XEvent = mem::zeroed();
                xlib::XNextEvent(self.dpy, &mut event);

                Some(event)
            } else {
                None
            }
        }
    }

    pub fn select_input(&mut self) {
        unsafe {
            let mut focused = 0;

            xlib::XGetInputFocus(self.dpy, &mut focused, &mut 0);

            if focused != self.window {
                xlib::XGrabKeyboard(self.dpy, focused, xlib::False, xlib::GrabModeAsync, xlib::GrabModeAsync, xlib::CurrentTime);
                xlib::XSelectInput(self.dpy, focused, xlib::KeyPressMask | xlib::KeyReleaseMask | xlib::ExposureMask);
            }
        }
    }

    pub fn keycode_to_keysym(&self, keycode: u8) -> u64 {
        unsafe {
            xlib::XKeycodeToKeysym(self.dpy, keycode, 0)
        }
    }

    pub fn keycode_to_string(&self, keycode: u8) -> Result<&str, Box<dyn std::error::Error>> {
        unsafe {
            Ok(CStr::from_ptr(xlib::XKeysymToString(self.keycode_to_keysym(keycode))).to_str()?)
        }
    }

    pub fn load_font(&mut self, font: &str) -> Result<*mut xlib::XFontStruct, Box<dyn std::error::Error>> {
        unsafe {
            let font = xlib::XLoadQueryFont(self.dpy, self.null_terminate(font).as_ptr() as *const i8);

            if font.is_null() {
                Err("BadAlloc".into())
            } else {
                Ok(font)
            }
        }
    }

    pub fn draw_rec(&mut self, x: i32, y: i32, width: u32, height: u32, color: Color) {
        unsafe {
            xlib::XSetForeground(self.dpy, self.gc, color.encode());
            xlib::XFillRectangle(self.dpy, self.window, self.gc, x, y, width, height);
        }
    }

    pub fn text_width(&self, text: &str, font: *mut xlib::XFontStruct) -> i32 {
        unsafe {
            xlib::XTextWidth(font, self.null_terminate(text).as_ptr() as *const i8, text.len() as i32)
        }
    }

    pub fn draw_text(&mut self, text: &str, x: i32, y: i32, font: *mut xlib::XFontStruct, color: Color) {
        unsafe {
            xlib::XSetFont(self.dpy, self.gc, (*font).fid);
            xlib::XSetForeground(self.dpy, self.gc, color.encode());
            xlib::XDrawString(self.dpy, self.window, self.gc, x, y, self.null_terminate(text).as_ptr() as *const i8, text.len() as i32);
        }
    }
}


