use std::io::{Cursor};
use std::path::{Path, PathBuf};

use bcmp::AlgoSpec;
use bindgen::callbacks::ParseCallbacks;
use clap::Parser;
use color_eyre::eyre::{Context, Result};
use regex::{self, Regex};

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "cec_bindgen")]
    src_path: String,
    #[arg(short, long)]
    major_version: String,
    #[arg(short, long)]
    dest_path: Option<String>,
}

struct CecVersion<'a> {
    major: u32,
    minor: u32,
    patch: u32,
    git_tag: &'a str,
}

impl From<&str> for CecVersion<'_> {
    fn from(major_version: &str) -> Self {
        match major_version {
            "4" => Self {
                major: 4,
                minor: 0,
                patch: 5,
                git_tag: "libcec-4.0.5",
            },
            "5" => Self {
                major: 5,
                minor: 0,
                patch: 0,
                git_tag: "libcec-5.0.0",
            },
            "6" => Self {
                major: 6,
                minor: 0,
                patch: 2,
                git_tag: "libcec-5.0.0",
            },
            _ => panic!("Unexpected major version"),
        }
    }
}
fn create_version_h<P: AsRef<Path>>(path: P, libcec_version_info: CecVersion<'_>) {
    let version_h_in = path.as_ref().join("include").join("version.h.in");
    let version_h = path.as_ref().join("include").join("version.h");

    let mut version_h_contents =
        std::fs::read_to_string(version_h_in).expect("Failed to read version.h.in");

    let re = Regex::new(r"@LIBCEC_VERSION_(MAJOR|MINOR|PATCH)@").unwrap();
    version_h_contents = re
        .replace_all(
            &version_h_contents,
            |captures: &regex::Captures| match &captures[0] {
                "@LIBCEC_VERSION_MAJOR@" => libcec_version_info.major.to_string(),
                "@LIBCEC_VERSION_MINOR@" => libcec_version_info.minor.to_string(),
                "@LIBCEC_VERSION_PATCH@" => libcec_version_info.patch.to_string(),
                _ => unreachable!(),
            },
        )
        .into();

    std::fs::write(version_h, version_h_contents).expect("Failed to create version.h");
}

fn fetch_libcec_source<P: AsRef<Path>>(path: P, major_version: &str) -> Result<()> {
    let libcec_version_info: CecVersion = major_version.into();
    let url = format!(
        "https://github.com/Pulse-Eight/libcec/archive/refs/tags/{}.zip",
        &libcec_version_info.git_tag
    );
    dbg!(major_version, &url);

    if !path.as_ref().exists() {
        let file = reqwest::blocking::get(&url)?
            .bytes()
            .context(format!("failed to download libcec from {url}"))?;
        zip_extract::extract(Cursor::new(file), path.as_ref(), true).context(format!(
            "failed to extract libcec archive to `{}`",
            path.as_ref().to_string_lossy()
        ))?;
    }

    create_version_h(path, libcec_version_info);

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let tmp_dir = tempfile::tempdir().context("failed to create temp directory")?;
    let build_path = tmp_dir.path();
    let src_path = PathBuf::from(args.src_path);
    let lib_path = build_path.join("libcec");
    let out_path = PathBuf::from(match args.dest_path {
        Some(x) => x,
        None => format!(
            "src/lib_abi{}_{}.rs",
            &args.major_version,
            target_lexicon::HOST
        ),
    });

    dbg!(&lib_path, &out_path, &tmp_dir, target_lexicon::HOST);

    // Only the headers are used, so fetch the release version since it's smaller.
    fetch_libcec_source(&lib_path, &args.major_version).context("failed to fetch libcec source")?;
    run_bindgen(&src_path, &lib_path, &out_path).context("failed to run bindgen")?;
    dbg!(&out_path);

    Ok(())
}

fn run_bindgen<P: AsRef<Path>>(src_path: P, lib_path: P, out_path: P) -> Result<()> {
    const ALLOW_REGEX: &str = "(libcec|cec|CEC|LIBCEC)_.*";
    let include_path = lib_path.as_ref().join("include");
    let header_path = src_path.as_ref().join("wrapper.h");

    let bindings = bindgen::Builder::default()
        .header(header_path.to_string_lossy())
        .allowlist_type(ALLOW_REGEX)
        .allowlist_function(ALLOW_REGEX)
        .allowlist_var(ALLOW_REGEX)
        .prepend_enum_name(false)
        .sort_semantically(true)
        .merge_extern_blocks(true)
        .derive_default(true)
        .derive_debug(true)
        .derive_copy(true)
        .clang_args([
            "--verbose",
            "--include-directory",
            &include_path.to_string_lossy(),
        ])
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(TidySymbols))
        .generate()
        .context("failed to generate bindings")?;

    bindings.write_to_file(out_path.as_ref()).context(format!(
        "failed to write bindings to `{}`",
        out_path.as_ref().to_string_lossy()
    ))?;

    Ok(())
}

#[derive(Debug)]
struct TidySymbols;

impl ParseCallbacks for TidySymbols {
    fn will_parse_macro(&self, _name: &str) -> bindgen::callbacks::MacroParsingBehavior {
        bindgen::callbacks::MacroParsingBehavior::Default
    }

    fn generated_name_override(
        &self,
        _item_info: bindgen::callbacks::ItemInfo<'_>,
    ) -> Option<String> {
        None
    }

    fn generated_link_name_override(
        &self,
        _item_info: bindgen::callbacks::ItemInfo<'_>,
    ) -> Option<String> {
        None
    }

    fn int_macro(&self, _name: &str, _value: i64) -> Option<bindgen::callbacks::IntKind> {
        None
    }

    fn enum_variant_behavior(
        &self,
        _enum_name: Option<&str>,
        _original_variant_name: &str,
        _variant_value: bindgen::callbacks::EnumVariantValue,
    ) -> Option<bindgen::callbacks::EnumVariantCustomBehavior> {
        None
    }

    fn enum_variant_name(
        &self,
        enum_name: Option<&str>,
        variant_name: &str,
        _value: bindgen::callbacks::EnumVariantValue,
    ) -> Option<String> {
        let exceptional_prefixes = [
            "CEC_AUDIO_RATE_",
            "CEC_AUDIO_",
            "ADAPTERTYPE_",
            "CEC_VENDOR_",
            "CEC_DEVICE_STATUS_",
            "CECDEVICE_",
        ];
        let exception = exceptional_prefixes
            .iter()
            .flat_map(|prefix| {
                variant_name
                    .strip_prefix(prefix)
                    .map(|variant| (prefix, variant))
            })
            .max_by(|(a, _), (b, _)| a.len().cmp(&b.len()));

        if let Some((_prefix, variant)) = exception {
            return Some(variant.to_owned());
        }

        let prefixes = ["enum ", "LIB"];
        let mut enum_name = enum_name.unwrap();
        for prefix in prefixes {
            if let Some(x) = enum_name.strip_prefix(prefix) {
                enum_name = x;
            }
        }
        let enum_name = enum_name.to_uppercase();

        let variant_name = variant_name.trim();
        let substring = bcmp::longest_common_substring(
            variant_name.as_bytes(),
            enum_name.as_bytes(),
            AlgoSpec::HashMatch(2),
        );

        let prefix = format!(
            "{}_",
            &variant_name[substring.first_pos..substring.first_end()]
        );

        if let Some(x) = variant_name.strip_prefix(&prefix) {
            if x.chars().next().unwrap().is_numeric() {
                Some(format!("_{x}"))
            } else {
                Some(x.to_string())
            }
        } else {
            None
        }
    }

    fn item_name(&self, _name: &str) -> Option<String> {
        None
    }

    fn blocklisted_type_implements_trait(
        &self,
        _name: &str,
        _derive_trait: bindgen::callbacks::DeriveTrait,
    ) -> Option<bindgen::callbacks::ImplementsTrait> {
        None
    }

    fn add_derives(&self, _info: &bindgen::callbacks::DeriveInfo<'_>) -> Vec<String> {
        vec![]
    }

    fn process_comment(&self, _comment: &str) -> Option<String> {
        None
    }

    fn str_macro(&self, _name: &str, _value: &[u8]) {}
    fn func_macro(&self, _name: &str, _value: &[&[u8]]) {}
    fn include_file(&self, _filename: &str) {}
    fn read_env_var(&self, _key: &str) {}
}
