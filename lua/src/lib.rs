extern crate rlua;

pub mod stub;

use std::rc::Rc;

use rlua::{Lua as RLua, Value, ToLua, UserData, UserDataMethods, Error as LuaError};
pub use rlua::{Result as LuaResult};

#[derive(Debug)]
pub enum IfaceError {
    ExitPlease,
}

impl std::fmt::Display for IfaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::error::Error;
        writeln!(f, "{}", self.description())
    }
}

impl std::error::Error for IfaceError {
    fn description(&self) -> &str {
        match *self {
            IfaceError::ExitPlease => "Lua should Exit",
        }
    }
}

impl From<IfaceError> for LuaError {
    fn from(err: IfaceError) -> Self {
        LuaError::external(err)
    }
}

pub type IfaceResult<T> = Result<T, IfaceError>;

pub struct Lua<T: LuaInterface> {
    lua: RLua,
    iface: Rc<T>,
}

pub enum Event {
    Stopped,
    NewGame,
}

impl<'lua> ToLua<'lua> for Event {
    fn to_lua(self, lua: &'lua RLua) -> LuaResult<Value<'lua>> {
        match self {
            Event::Stopped => "stopped".to_lua(lua),
            Event::NewGame => "newgame".to_lua(lua),
        }
    }
}

pub trait LuaInterface {
    fn step(&self) -> IfaceResult<Event>;
    fn press_key(&self, key: String) -> IfaceResult<()>;
    fn release_key(&self, key: String) -> IfaceResult<()>;
    fn move_mouse(&self, x: i32, y: i32) -> IfaceResult<()>;
    fn get_delta(&self) -> IfaceResult<f64>;
    fn set_delta(&self, delta: f64) -> IfaceResult<()>;
    fn get_location(&self) -> IfaceResult<(f32, f32, f32)>;
    fn set_location(&self, x: f32, y: f32, z: f32) -> IfaceResult<()>;
    fn get_rotation(&self) -> IfaceResult<(f32, f32, f32)>;
    fn set_rotation(&self, pitch: f32, yaw: f32, roll: f32) -> IfaceResult<()>;
    fn get_velocity(&self) -> IfaceResult<(f32, f32, f32)>;
    fn set_velocity(&self, x: f32, y: f32, z: f32) -> IfaceResult<()>;
    fn get_acceleration(&self) -> IfaceResult<(f32, f32, f32)>;
    fn set_acceleration(&self, x: f32, y: f32, z: f32) -> IfaceResult<()>;
    fn wait_for_new_game(&self) -> IfaceResult<()>;

    fn print(&self, s: String) -> IfaceResult<()>;
}

struct Wrapper<T>(T);

impl<T> std::ops::Deref for Wrapper<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Wrapper<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: 'static + LuaInterface> UserData for Wrapper<Rc<T>> {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("step", |_, this, _: ()| {
            Ok(this.step()?)
        });
        methods.add_method("press_key", |_, this, key: String| {
            Ok(this.press_key(key)?)
        });
        methods.add_method("release_key", |_, this, key: String| {
            Ok(this.release_key(key)?)
        });
        methods.add_method("move_mouse", |_, this, (x, y): (i32, i32)| {
            Ok(this.move_mouse(x, y)?)
        });
        methods.add_method("get_delta", |_, this, _: ()| {
            Ok(this.get_delta()?)
        });
        methods.add_method("set_delta", |_, this, delta: f64| {
            Ok(this.set_delta(delta)?)
        });
        methods.add_method("get_location", |_, this, _: ()| {
            Ok(this.get_location()?)
        });
        methods.add_method("set_location", |_, this, (x, y, z): (f32, f32, f32)| {
            Ok(this.set_location(x, y, z)?)
        });
        methods.add_method("get_rotation", |_, this, _: ()| {
            Ok(this.get_rotation()?)
        });
        methods.add_method("set_rotation", |_, this, (pitch, yaw, roll): (f32, f32, f32)| {
            Ok(this.set_rotation(pitch, yaw, roll)?)
        });
        methods.add_method("get_velocity", |_, this, _: ()| {
            Ok(this.get_velocity()?)
        });
        methods.add_method("set_velocity", |_, this, (x, y, z): (f32, f32, f32)| {
            Ok(this.set_velocity(x, y, z)?)
        });
        methods.add_method("get_acceleration", |_, this, _: ()| {
            Ok(this.get_acceleration()?)
        });
        methods.add_method("set_acceleration", |_, this, (x, y, z): (f32, f32, f32)| {
            Ok(this.set_acceleration(x, y, z)?)
        });
        methods.add_method("wait_for_new_game", |_, this, _: ()| {
            Ok(this.wait_for_new_game()?)
        });

        methods.add_method("print", |_, this, s: String| {
            Ok(this.print(s)?)
        })
    }
}

impl<T: LuaInterface + 'static> Lua<T> {
    pub fn new(iface: Rc<T>) -> Lua<T> {
        let lua = RLua::new();
        Lua {
            lua,
            iface,
        }
    }

    pub fn execute(&mut self, code: &str) -> LuaResult<()> {
        self.lua.scope(|scope| {
            let iface = scope.create_userdata(Wrapper(self.iface.clone()))?;
            self.lua.globals().set("tas", iface)?;
            let function = self.lua.load(code, None)?;
            function.call::<_, ()>(())
        })
    }
}
