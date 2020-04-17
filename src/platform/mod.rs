pub use self::os::*;

#[cfg(target_os = "macos")]
#[path="macos.rs"]
mod os;

#[cfg(all(not(target_os = "macos")))]
compile_error!("The platform you're compiling for is not supported by webview");
