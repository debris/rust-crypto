// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate gcc;

use std::env;
use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
const OS: &'static str = "darwin";
#[cfg(target_os = "linux")]
const OS: &'static str = "linux";
#[cfg(target_os = "windows")]
const OS: &'static str = "windows";

const ANDROID_INCLUDE: &'static str = "platforms/android-21/arch-arm64/usr/include";

fn android_aarch_compiler() -> String {
	"toolchains/aarch64-linux-android-4.9/prebuilt/".to_owned() + OS + "-x86_64/bin"
}

fn android_arm_compiler() -> String {
	"toolchains/arm-linux-androideabi-4.9/prebuilt/".to_owned() + OS + "-x86_64/bin"
}

fn android_i686_compiler() -> String {
	"toolchains/x86-4.9/prebuilt/".to_owned() + OS + "-x86_64/bin"
}

fn concat_paths(first: &str, second: &str) -> PathBuf {
    let mut path = PathBuf::from(first);
    path.push(second);
    path
}

fn setup_android(config: &mut gcc::Config) {
    let path = env::var_os("PATH").unwrap();
    let ndk_home = env::var("NDK_HOME").expect("NDK_HOME is not set");
    let mut paths = env::split_paths(&path).collect::<Vec<_>>();
    paths.push(concat_paths(&ndk_home, &android_aarch_compiler()));
    paths.push(concat_paths(&ndk_home, &android_arm_compiler()));
    paths.push(concat_paths(&ndk_home, &android_i686_compiler()));

    let new_path = env::join_paths(paths).unwrap();
    env::set_var("PATH", new_path);

    config.include(&concat_paths(&ndk_home, ANDROID_INCLUDE));
}

fn main() {
    let target = env::var("TARGET").unwrap();
    let host = env::var("HOST").unwrap();
    if target.contains("msvc") && host.contains("windows") {
        let mut config = gcc::Config::new();
        config.file("src/util_helpers.asm");
        config.file("src/aesni_helpers.asm");
        if target.contains("x86_64") {
            config.define("X64", None);
        }
        config.compile("lib_rust_crypto_helpers.a");
    }
    else {
        let mut cfg = gcc::Config::new();
        cfg.file("src/util_helpers.c");
        cfg.file("src/aesni_helpers.c");
        if target.contains("android") {
            setup_android(&mut cfg);
        }

        if env::var_os("CC").is_none() {
            if host.contains("openbsd") {
                // Use clang on openbsd since there have been reports that
                // GCC doesn't like some of the assembly that we use on that
                // platform.
                cfg.compiler(Path::new("clang"));
            } else if target == host {
                cfg.compiler(Path::new("cc"));
            }
        }
        cfg.compile("lib_rust_crypto_helpers.a");
    }
}

