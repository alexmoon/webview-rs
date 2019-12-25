pub use self::platform::*;

#[cfg(target_os = "macos")]
#[path="macos.rs"]
mod platform;

#[cfg(all(not(target_os = "macos")))]
compile_error!("The platform you're compiling for is not supported by winit");
