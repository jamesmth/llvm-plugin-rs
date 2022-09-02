fn main() {
    let includedir = llvm_sys::llvm_config("--includedir");

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .include(includedir.trim())
        .file("cpp/ffi.cc");

    #[cfg(target_env = "msvc")]
    build.flag_if_supported("/std:c++17");
    #[cfg(not(target_env = "msvc"))]
    build.flag_if_supported("-std=c++17");

    let has_rtti = llvm_sys::llvm_config("--has-rtti");
    if has_rtti.trim() == "YES" {
        #[cfg(target_env = "msvc")]
        build.flag_if_supported("/GR-");
        #[cfg(not(target_env = "msvc"))]
        build.flag_if_supported("-fno-rtti");
    }

    build.warnings(false);
    build.compile("llvm-plugin-cpp");

    #[cfg(target_os = "linux")]
    {
        let libdir = llvm_sys::llvm_config("--libdir");
        println!("cargo:rustc-link-search=native={}", libdir.trim());
        println!("cargo:rustc-link-lib=dylib=LLVM");
    }

    #[cfg(target_os = "windows")]
    {
        let libdir = llvm_sys::llvm_config("--libdir");
        println!("cargo:rustc-link-search=native={}", libdir.trim());
        println!("cargo:rustc-link-lib=dylib=LLVM-C");
        println!("cargo:rustc-link-lib=dylib=opt");
    }

    println!("cargo:rerun-if-changed=cpp");
}

// Most code from this module was taken from the `llvm-sys` crate.
// Locally patching such crate wasn't an option, because inkwell requires
// it to be published (for version selection).
//
// Like `llvm-sys`, we need to locate the `llvm-config` binary, which
// is why we borrowed code from this crate.
mod llvm_sys {
    use lazy_static::lazy_static;

    use regex::Regex;
    use semver::Version;
    use std::env;
    use std::ffi::OsStr;
    use std::io::{self, ErrorKind};
    use std::path::PathBuf;
    use std::process::Command;

    /// A single path to search for LLVM in (containing bin/llvm-config)
    const ENV_LLVM_PREFIX: &str = "LLVM_PLUGIN_PREFIX";

    lazy_static! {
        /// Filesystem path to an llvm-config binary for the correct version.
        static ref LLVM_CONFIG_PATH: PathBuf = locate_llvm_config();
    }

    /// Try to find a version of llvm-config that is compatible with this crate.
    ///
    /// If $LLVM_PLUGIN_PREFIX is set, look for llvm-config ONLY in there. The assumption is
    /// that the user know best, and they want to link to a specific build or fork of LLVM.
    ///
    /// If $LLVM_PLUGIN_PREFIX is NOT set, then look for llvm-config in $PATH.
    ///
    /// Returns None on failure.
    fn locate_llvm_config() -> PathBuf {
        let prefix = env::var_os(ENV_LLVM_PREFIX)
            .map(|p| PathBuf::from(p).join("bin"))
            .unwrap_or_else(PathBuf::new);
        for binary_name in llvm_config_binary_names() {
            let binary_name = prefix.join(binary_name);
            match llvm_version(&binary_name) {
                // we don't need strict LLVM version checking, `llvm-sys` already
                // does it for us
                Ok(_) => {
                    // Compatible version found. Nice.
                    return binary_name;
                }
                Err(ref e) if e.kind() == ErrorKind::NotFound => {
                    // Looks like we failed to execute any llvm-config. Keep
                    // searching.
                }
                // Some other error, probably a weird failure. Give up.
                Err(e) => panic!("Failed to search PATH for llvm-config: {}", e),
            }
        }

        panic!("llvm-config not found")
    }

    /// Return an iterator over possible names for the llvm-config binary.
    fn llvm_config_binary_names() -> std::vec::IntoIter<String> {
        let (major, minor) = if cfg!(feature = "llvm10-0") {
            (10, 0)
        } else if cfg!(feature = "llvm11-0") {
            (11, 0)
        } else if cfg!(feature = "llvm12-0") {
            (12, 0)
        } else if cfg!(feature = "llvm13-0") {
            (13, 0)
        } else if cfg!(feature = "llvm14-0") {
            (14, 0)
        } else {
            panic!("Missing llvm* feature");
        };

        let mut base_names = vec![
            "llvm-config".into(),
            format!("llvm-config-{}", major),
            format!("llvm{}-config", major),
            format!("llvm-config-{}.{}", major, minor),
            format!("llvm-config{}{}", major, minor),
        ];

        // On Windows, also search for llvm-config.exe
        if cfg!(windows) {
            let mut exe_names = base_names.clone();
            for name in exe_names.iter_mut() {
                name.push_str(".exe");
            }
            base_names.extend(exe_names);
        }

        base_names.into_iter()
    }

    /// Get the output from running `llvm-config` with the given argument.
    ///
    /// Lazily searches for or compiles LLVM as configured by the environment
    /// variables.
    pub fn llvm_config(arg: &str) -> String {
        llvm_config_ex(&*LLVM_CONFIG_PATH.clone(), arg)
            .expect("Surprising failure from llvm-config")
    }

    /// Invoke the specified binary as llvm-config.
    ///
    /// Explicit version of the `llvm_config` function that bubbles errors
    /// up.
    fn llvm_config_ex<S: AsRef<OsStr>>(binary: S, arg: &str) -> io::Result<String> {
        Command::new(binary).arg(arg).output().and_then(|output| {
            if output.stdout.is_empty() {
                Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "llvm-config returned empty output",
                ))
            } else {
                Ok(String::from_utf8(output.stdout)
                    .expect("Output from llvm-config was not valid UTF-8"))
            }
        })
    }

    /// Get the LLVM version using llvm-config.
    fn llvm_version<S: AsRef<OsStr>>(binary: &S) -> io::Result<Version> {
        let version_str = llvm_config_ex(binary.as_ref(), "--version")?;

        // LLVM isn't really semver and uses version suffixes to build
        // version strings like '3.8.0svn', so limit what we try to parse
        // to only the numeric bits.
        let re = Regex::new(r"^(?P<major>\d+)\.(?P<minor>\d+)(?:\.(?P<patch>\d+))??").unwrap();
        let c = match re.captures(&version_str) {
            Some(c) => c,
            None => {
                panic!(
                    "Could not determine LLVM version from llvm-config. Version string: {}",
                    version_str
                );
            }
        };

        // some systems don't have a patch number but Version wants it so we just append .0 if it isn't
        // there
        let s = match c.name("patch") {
            None => format!("{}.0", &c[0]),
            Some(_) => c[0].to_string(),
        };
        Ok(Version::parse(&s).unwrap())
    }
}