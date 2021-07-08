#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate num;
#[macro_use]
extern crate num_derive;

extern crate num_traits;

use std::{
    alloc::{self, Layout},
    cmp,
    ffi::CString,
    io::Cursor,
    mem,
    sync::{Arc, Mutex},
};
use std::{
    convert::{TryFrom, TryInto},
    ffi::CStr,
    panic,
    time::Duration,
};
use tinybmp::{Bmp, FileType, Header, Pixel};
pub mod c_api;
use c_api::{ibitmap, ifont};

#[derive(Debug, Copy, Clone)]
pub struct Color(pub i32);

impl Color {
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self((blue as i32) << 16 + (green as i32) << 8 + red as i32)
    }

    pub const fn gs(intensity: u8) -> Self {
        Self(intensity as i32 * 0x010101)
    }

    pub const WHITE: Self = Self(c_api::WHITE);
    pub const LGRAY: Self = Self(c_api::LGRAY);
    pub const DGRAY: Self = Self(c_api::DGRAY);
    pub const BLACK: Self = Self(c_api::BLACK);
}


pub struct PbBmp(pub *mut ibitmap);

impl PbBmp {
    pub fn get_pointer(&self) -> *mut ibitmap{
        self.0
    }
}

impl From<Bmp<'_>> for PbBmp {
    fn from(bmp: Bmp<'_>) -> Self {
        let bmp_width = bmp.header.image_width;
        let bmp_height = bmp.header.image_height;
        let pixels = bmp.image_data();

        let layout = Layout::from_size_align(
            mem::size_of::<ibitmap>() + pixels.len() * mem::size_of::<u8>(),
            cmp::max(mem::align_of::<u8>(), mem::align_of::<ibitmap>()),
        )
        .unwrap();

        let bmp_struct = unsafe { alloc::alloc(layout) } as *mut ibitmap;
        unsafe {
            (*bmp_struct).width = bmp_width as u16;
            (*bmp_struct).height = bmp_height as u16;
            (*bmp_struct).depth = bmp.header.bpp;
            (*bmp_struct).scanline = bmp.header.image_data_len as u16 / bmp_width as u16;
            (*bmp_struct)
                .data
                .as_mut_ptr()
                .copy_from_nonoverlapping(pixels.as_ptr(), pixels.len());
        }
        PbBmp(bmp_struct)
    }
}   


#[macro_export]
macro_rules! include_bmp {
    ($expression:expr) => {{
        Bmp::from_slice(include_bytes!($expression)).expect("Failed to parse BMP image")
        
    }};
}

pub fn scale_bitmap_to(bmp: *mut ibitmap, w: i32, h: i32) -> *mut ibitmap {
    unsafe { c_api::BitmapStretchCopy(bmp, 0, 0, (*bmp).width as i32, (*bmp).height as i32, w, h) }
}


pub fn mirror_bitmap(bmp: *mut ibitmap, mirror_flags: i32) {
    unsafe { c_api::MirrorBitmap(bmp, mirror_flags) }
}

pub trait EventHandler {
    fn handle_event(&mut self, event: c_api::Event, par1: i32, par2: i32) -> i32;
}

static mut iv_event_handler: Option<Arc<Mutex<dyn EventHandler>>> = None;
extern "C" fn iv_event_handler_wrapper(event: i32, par1: i32, par2: i32) -> i32 {
    unsafe {
        match iv_event_handler {
            Some(ref mut event_handler) => {
                if let Some(event) = num::FromPrimitive::from_i32(event) {
                    let result = panic::catch_unwind(|| {
                        event_handler
                            .lock()
                            .expect("Event handler is locked")
                            .handle_event(event, par1, par2)
                    });

                    match result {
                        Ok(v) => v,
                        Err(err) => {
                            message(c_api::Icon::ERROR, "Panic", &format!("{:?}", err), 10000);

                            std::thread::sleep(Duration::from_secs(10));
                            -2
                        }
                    }
                } else {
                    -1
                }
            }
            None => -2,
        }
    }
}

pub fn main(event_handler: &Arc<Mutex<dyn EventHandler>>) {
    unsafe {
        iv_event_handler = Some(Arc::clone(event_handler));
        c_api::InkViewMain(Some(iv_event_handler_wrapper));
    }
}
/// Put Event::EXIT into applications event queue and closes the application.
pub fn exit() {
    unsafe {
        c_api::CloseApp();
    }
}

/// Put Event::SHOW into applications event queue.
pub fn repaint() {
    unsafe {
        c_api::CloseApp();
    }
}

////////////////////////////////////////////////////////////////////////////////
// Graphic functions

pub type Dither = c_api::Dither;

pub fn screen_width() -> i32 {
    unsafe { c_api::ScreenWidth() }
}

pub fn screen_height() -> i32 {
    unsafe { c_api::ScreenHeight() }
}

#[repr(i32)]
pub enum Orientation{
    Portrait = 0,
    Landscape90 = 1,
    Landscape270 = 2,
    Portrait180 = 3,
    Auto = -1
}

pub fn set_orientation(orientation: Orientation){
    unsafe{
        c_api::SetOrientation(orientation as i32)
    }
}

pub fn get_orientation() -> Orientation{
    let orientation = unsafe{
        c_api::GetOrientation() 
    };
    
    todo!()
}


pub fn panel_height() -> i32 {
    unsafe { c_api::PanelHeight() }
}

pub fn clear_screen() {
    unsafe {
        c_api::ClearScreen();
    }
}

pub fn set_clip(x: i32, y: i32, w: i32, h: i32) {
    unsafe {
        c_api::SetClip(x, y, w, h);
    }
}

pub fn draw_pixel(x: i32, y: i32, color: Color) {
    unsafe {
        c_api::DrawPixel(x, y, color.0);
    }
}

pub fn draw_line(x1: i32, y1: i32, x2: i32, y2: i32, color: Color) {
    unsafe {
        c_api::DrawLine(x1, y1, x2, y2, color.0);
    }
}

pub fn draw_dot_line(x1: i32, y1: i32, x2: i32, y2: i32, color: Color, step: i32) {
    unsafe {
        c_api::DrawLineEx(x1, y1, x2, y2, color.0, step);
    }
}

#[cfg(feature = "sdk_v6")]
pub fn draw_dash_line(x1: i32, y1: i32, x2: i32, y2: i32, color: Color, fill: u32, space: u32) {
    unsafe {
        c_api::DrawDashLine(x1, y1, x2, y2, color.0, fill, space);
    }
}

pub fn draw_rect(x: i32, y: i32, w: i32, h: i32, color: Color) {
    unsafe {
        c_api::DrawRect(x, y, w, h, color.0);
    }
}

pub fn draw_rect_round(x: i32, y: i32, w: i32, h: i32, color: Color, radius: i32) {
    unsafe {
        c_api::DrawRectRound(x, y, w, h, color.0, radius);
    }
}

pub fn fill_area(x: i32, y: i32, w: i32, h: i32, color: Color) {
    unsafe {
        c_api::FillArea(x, y, w, h, color.0);
    }
}

pub fn invert_area(x: i32, y: i32, w: i32, h: i32) {
    unsafe {
        c_api::InvertArea(x, y, w, h);
    }
}

pub fn invert_area_bw(x: i32, y: i32, w: i32, h: i32) {
    unsafe {
        c_api::InvertAreaBW(x, y, w, h);
    }
}

pub fn dim_area(x: i32, y: i32, w: i32, h: i32, color: Color) {
    unsafe {
        c_api::DimArea(x, y, w, h, color.0);
    }
}

pub fn draw_selection(x: i32, y: i32, w: i32, h: i32, color: Color) {
    unsafe {
        c_api::DrawSelection(x, y, w, h, color.0);
    }
}

pub fn draw_circle(x: i32, y: i32, radius: i32, color: Color) {
    unsafe {
        c_api::DrawCircle(x, y, radius, color.0);
    }
}

pub fn draw_bitmap(x: i32, y: i32, bmp: *mut ibitmap) {
    unsafe {
        c_api::DrawBitmap(
            x,
            y,
            bmp,
        );
    }
}

// Выделение текста
pub fn draw_pick_out(x: i32, y: i32, w: i32, h: i32, key: &str) {
    let c_key = CString::new(key).expect("CString::new failed").into_raw();
    unsafe {
        c_api::DrawPickOut(x, y, w, h, c_key);
    }
}

pub fn dither_area(x: i32, y: i32, w: i32, h: i32, levels: i32, method: Dither) {
    unsafe {
        c_api::DitherArea(x, y, w, h, levels, method as i32);
    }
}

pub fn dither_area_quick_2level(x: i32, y: i32, w: i32, h: i32) {
    unsafe {
        c_api::DitherAreaQuick2Level(x, y, w, h);
    }
}

#[cfg(feature = "sdk_v6")]
pub fn dither_area_pattern_2level(x: i32, y: i32, w: i32, h: i32) {
    unsafe {
        c_api::DitherAreaPattern2Level(x, y, w, h);
    }
}

pub fn draw_diagonal_hatch(x: i32, y: i32, w: i32, h: i32, step: i32, color: Color) {
    unsafe {
        c_api::DrawDiagonalHatch(x, y, w, h, step, color.0);
    }
}

pub fn transparent(x: i32, y: i32, w: i32, h: i32, percent: i32) {
    unsafe {
        c_api::Transparent(x, y, w, h, percent);
    }
}


#[derive(Clone, Copy)]
pub struct Font(pub *mut ifont);

// Text functions
pub fn open_font(name: &str, size: i32, aa: i32) -> Font {
    Font(unsafe { c_api::OpenFont(CString::new(name).unwrap().into_raw(), size, aa) })
}

pub fn set_font(font: Font, color: Color) {
    unsafe { c_api::SetFont(font.0, color.0.into()) }
}

pub fn text_rect_height(width: i32, string: &str, flags: i32) -> i32{
    return unsafe { c_api::TextRectHeight(width, CString::new(string).unwrap().into_raw(), flags) }
}


pub fn draw_text_rect(x: i32, y: i32, w: i32, h: i32, s: &str, flags: i32) -> String {
    unsafe {
        CStr::from_ptr(c_api::DrawTextRect(
            x,
            y,
            w,
            h,
            CString::new(s).unwrap().into_raw(),
            flags,
        ))
        .to_string_lossy()
        .into()
    }
}

////////////////////////////////////////////////////////////////////////////////
// Screen update

pub fn full_update() {
    unsafe {
        c_api::FullUpdate();
    }
}

pub fn soft_update() {
    unsafe {
        c_api::FullUpdate();
    }
}

pub fn partial_update(x: i32, y: i32, w: i32, h: i32) {
    unsafe {
        c_api::PartialUpdate(x, y, w, h);
    }
}

////////////////////////////////////////////////////////////////////////////////
// UI functions

//pub type Icon = c_api::Icon;                /// Dialog icons
//pub type Button = c_api::Button;            /// Dialog buttons
pub type PanelType = c_api::PanelType;
/// InkView header panel control flags

pub fn panel_type() -> PanelType {
    unsafe {
        {
            c_api::GetPanelType().try_into().unwrap()
        }
    }
}

pub fn set_panel_type(panel_type: PanelType) {
    unsafe {
        c_api::SetPanelType(panel_type.0 as i32);
    }
}

pub fn message(icon: c_api::Icon, title: &str, text: &str, timeout: i32) {
    unsafe {
        c_api::Message(
            icon as i32,
            CString::new(title).unwrap().into_raw(),
            CString::new(text).unwrap().into_raw(),
            timeout,
        )
    }
}


#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, FromPrimitive)]
pub enum TextAlignFlag{
    ALIGN_LEFT = c_api::ALIGN_LEFT,
    ALIGN_CENTER = c_api::ALIGN_CENTER,
    ALIGN_RIGHT = c_api::ALIGN_RIGHT,
    ALIGN_FIT = c_api::ALIGN_FIT,
    VALIGN_TOP = c_api::VALIGN_TOP,
    VALIGN_MIDDLE = c_api::VALIGN_MIDDLE,
    VALIGN_BOTTOM = c_api::VALIGN_BOTTOM
}
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, FromPrimitive)]
pub enum MirrorFlag{
	X_MIRROR = c_api::XMIRROR,
	Y_MIRROR = c_api::YMIRROR
}

/*bitflags! {
    struct TextAlignFlag: i32 {
    const ALIGN_LEFT = c_api::ALIGN_LEFT;
    const ALIGN_CENTER = c_api::ALIGN_CENTER;
    const ALIGN_RIGHT = c_api::ALIGN_RIGHT;
    const ALIGN_FIT = c_api::ALIGN_FIT;
    const VALIGN_TOP = c_api::VALIGN_TOP;
    const VALIGN_MIDDLE = c_api::VALIGN_MIDDLE;
    const VALIGN_BOTTOM = c_api::VALIGN_BOTTOM;
    }
}


bitflags! {
    struct MirrorFlag: i32 {
    const X_MIRROR = c_api::XMIRROR;
	const Y_MIRROR = c_api::YMIRROR;
    }
}*/
