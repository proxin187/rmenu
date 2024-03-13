use crate::config::Config;

use x11::xrender;
use x11::xlib;
use x11::xft;

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

    pub fn hex(&self) -> String {
        format!("#{:x}{:x}{:x}", self.r, self.g, self.b)
    }
}

pub struct Display {
    dpy: *mut xlib::_XDisplay,
    gc: *mut xlib::_XGC,
    draw: *mut x11::xft::XftDraw,

    window: u64,
    screen: i32,

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
    pub fn open(config: &Config) -> Result<Display, Box<dyn std::error::Error>> {
        let dpy = unsafe { xlib::XOpenDisplay(ptr::null()) };

        if dpy.is_null() {
            Err("failed to open display".into())
        } else {
            unsafe {
                let mut values: xlib::XGCValues = mem::zeroed();
                let mut current_monitor = x11::xinerama::XineramaScreenInfo {
                    screen_number: 0,
                    x_org: 0,
                    y_org: 0,
                    width: 0,
                    height: 0,
                };

                let mut root_return = xlib::XDefaultRootWindow(dpy);
                let mut root_x = 0;

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

                // get current monitor
                {
                    let mut count = 0;

                    let monitors = x11::xinerama::XineramaQueryScreens(dpy, &mut count);

                    for i in 0..count {
                        let monitor = *monitors.offset(i as isize);

                        let range = monitor.x_org as i32..monitor.width as i32 + monitor.x_org as i32;

                        if range.contains(&root_x) {
                            current_monitor = monitor;
                        }
                    }
                }

                let bg = config.background.encode();
                let window = xlib::XCreateSimpleWindow(
                    dpy,
                    xlib::XDefaultRootWindow(dpy),
                    current_monitor.x_org as i32,
                    current_monitor.y_org as i32,
                    current_monitor.width as u32,
                    22,
                    0,
                    bg,
                    bg
                );
                let screen = xlib::XDefaultScreen(dpy);

                let gc = xlib::XCreateGC(dpy, window, 0, &mut values);
                let draw = xft::XftDrawCreate(dpy, window, xlib::XDefaultVisual(dpy, screen), xlib::XDefaultColormap(dpy, screen));

                xlib::XSync(dpy, xlib::False);

                Ok(Display {
                    dpy,
                    gc,
                    window,
                    screen,
                    draw,

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

    pub fn set_window_name(&mut self, name: &str) {
        unsafe {
            xlib::XStoreName(self.dpy, self.window, self.null_terminate(name).as_ptr() as *const i8);
        }
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

    pub fn xft_draw_string(
        &mut self,
        text: &str,
        x: i32,
        y: i32,
        font: *mut xft::XftFont,
        color: *const xft::XftColor,
    ) {
        unsafe {
            xft::XftDrawStringUtf8(self.draw, color, font, x, y, self.null_terminate(text).as_ptr(), text.len() as i32);
        }
    }

    pub fn xft_measure_string(&self, text: &str, font: *mut xft::XftFont) -> xrender::_XGlyphInfo {
        unsafe {
            let mut extents: xrender::_XGlyphInfo = mem::zeroed();

            xft::XftTextExtentsUtf8(self.dpy, font, self.null_terminate(text).as_ptr(), text.len() as i32, &mut extents);

            extents
        }
    }

    pub fn xft_color_alloc_name(&mut self, rgb: Color) -> Result<xft::XftColor, Box<dyn std::error::Error>> {
        let hex = rgb.hex();

        unsafe {
            let mut color: xft::XftColor = mem::zeroed();

            let result = xft::XftColorAllocName(
                self.dpy,
                xlib::XDefaultVisual(self.dpy, self.screen),
                xlib::XDefaultColormap(self.dpy, self.screen),
                self.null_terminate(&hex).as_ptr() as *const i8,
                &mut color,
            );

            if result == 0 {
                Err("XftColorAllocName failed".into())
            } else {
                Ok(color)
            }
        }
    }

    pub fn load_font(&mut self, font: &str) -> Result<*mut xft::XftFont, Box<dyn std::error::Error>> {
        unsafe {
            let font = xft::XftFontOpenName(self.dpy, self.screen, self.null_terminate(font).as_ptr() as *const i8);

            if font.is_null() {
                Err("XftFontOpenName failed".into())
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
}


