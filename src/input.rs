use std::sync::atomic::AtomicI32;

use dashmap::DashMap;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::sync::atomic::Ordering::SeqCst;
use std::ops::Deref;

use crate::time;

lazy_static!{
    static ref  KEYMAP:KeyMap = KeyMap::new();
    static ref MOUSE:Mouse = Mouse::new();
    static ref COOLDOWN_MAP:DashMap<Keycode,f32> = DashMap::new();
}

struct Mouse {
    dx:AtomicI32,
    dy:AtomicI32,
}
impl Mouse {
    fn new() -> Self{
        Mouse {
            dx:AtomicI32::new(0),
            dy:AtomicI32::new(0),
        }
    }
    fn store_motion(&self,x:i32,y:i32) {
        self.dx.fetch_add(x, SeqCst);
        self.dy.fetch_add(y, SeqCst);
    }
}
struct KeyMap {
    inner:DashMap<Keycode,bool>
}
impl KeyMap {
    fn new() -> Self {
        Self {inner:DashMap::new()}
    }
}
impl Deref for KeyMap {
    type Target = DashMap<Keycode,bool>;
    fn deref(&self) -> &DashMap<Keycode,bool>{
        &self.inner
    }
}


pub fn fetch_motion() -> (i32,i32) {
    let dx = MOUSE.dx.swap(0,SeqCst);
    let dy = MOUSE.dy.swap(0,SeqCst);
    (dx,dy)
}
pub fn get_key(
    keycode:Keycode,
    default:bool
) -> bool {
    let pair = KEYMAP.inner.get(&keycode);
    match pair {
        None => return default,
        Some( pair) =>  return *pair
    }
}
pub fn get_key_with_cooldown( 
    keycode:Keycode,
    default:bool,
    cooltime:f32
) -> bool{
    let tv = get_key(keycode, default);
    if !tv {return false;}
    if !COOLDOWN_MAP.contains_key(&keycode) {
        //不含该键，说明第一次按下该键
        COOLDOWN_MAP.insert(keycode, 0.0);
        return true;
    } else {
        let mut last = COOLDOWN_MAP.get_mut(&keycode).unwrap();
        let now = time::get_now();
        if (now - *last) > cooltime {
            *last = now;
            return true;
        }else {
            return false;
        }
    }
}
pub fn handle_sdl_input(event:&Event){

    match event {
        Event::MouseMotion {
            timestamp:_,window_id:_, which:_, mousestate:_,
            x:_,y:_,xrel, yrel
        } => {
            MOUSE.store_motion(*xrel, -*yrel);
        },
        Event::KeyDown {
            timestamp:_, window_id: _,
            keycode, scancode: _,
            keymod: _, repeat:_
        } => {
            if let Some(keycode) = keycode {
                KEYMAP.insert(keycode.clone(), true);
            }
        }
        Event::KeyUp {
            timestamp:_, window_id: _,
            keycode, scancode: _,
            keymod: _, repeat:_
        } => {
            if let Some(keycode) = keycode {
                KEYMAP.insert(keycode.clone(), false);
            }
        },
        _ => {}
    }
}