#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::fmt::Debug;

mod cpu;
#[cfg(any(target_os = "windows", target_os = "linux"))]
mod cuda;

#[derive(Default, Debug)]
pub struct MemInfo {
    total: u64,
    free: u64,
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum CPUCapability {
    None,
    Avx,
    Avx2,
}

impl CPUCapability {
    pub fn from(vv: &str) -> Self {
        match vv {
            "avx" => Self::Avx,
            "avx2" => Self::Avx2,
            "" => Self::None,
            _ => {
                unreachable!()
            }
        }
    }
}

impl Default for CPUCapability {
    fn default() -> Self {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if ::std::is_x86_feature_detected!("avx2") {
            Self::Avx2
        } else if ::std::is_x86_feature_detected!("avx") {
            Self::Avx
        } else {
            Self::None
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        Self::None
    }
}

#[derive(Default, Debug)]
pub struct DeviceInfo {
    pub memInfo: MemInfo,
    pub library: &'static str,
    pub variant: CPUCapability,
    pub minimum_memory: u64,
    pub dependency_paths: Vec<std::path::PathBuf>,
    pub env_workarounds: Vec<(String, String)>,
    pub id: String,
    pub name: String,
    pub compute: String,
    pub driver_version: DriverVersion,
}

impl DeviceInfo {
    pub(crate) fn variants(&self, vars: &Vec<Variant>) -> Vec<Variant> {
        vars.iter()
            .filter(|v| {
                if self.library == "cpu" {
                    v.library == "cpu" && self.variant >= CPUCapability::from(&v.variant)
                } else {
                    self.library == v.library || v.library == "cpu"
                }
            })
            .map(|v| v.clone())
            .collect()
    }
}

#[derive(Default, Debug)]
pub struct DriverVersion {
    pub major: i32,
    pub minor: i32,
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
struct CudaHandles {
    device_count: usize,
    cudart: Option<cuda::cudart::CudartHandle>,
    nvcuda: Option<cuda::nvcuda::NvCudaHandle>,
    _nvml: Option<cuda::nvml::NvMlHandle>,
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
impl CudaHandles {
    pub fn new() -> Result<Self> {
        let nvml = match cuda::nvml::NvMlHandle::new() {
            Ok(h) => Some(h),
            Err(e) => {
                log::warn!("{e}");
                None
            }
        };
        let (device_count, nvcuda) = match cuda::nvcuda::NvCudaHandle::new() {
            Ok((d, h)) => (d, Some(h)),
            Err(e) => {
                log::warn!("{e}");
                (0, None)
            }
        };
        let (device_count1, cudart) = match cuda::cudart::CudartHandle::new() {
            Ok((d, h)) => (d, Some(h)),
            Err(e) => {
                log::warn!("{e}");
                (0, None)
            }
        };
        let device_count = if device_count == 0 {
            device_count1
        } else {
            device_count
        };
        if device_count > 0 {
            Ok(Self {
                _nvml: nvml,
                device_count,
                nvcuda,
                cudart,
            })
        } else {
            Err(Error::CudaNotFound)
        }
    }
    pub fn get_devices_info(&self) -> Vec<DeviceInfo> {
        let mut res = vec![];
        for device in 0..self.device_count {
            let _meminfo = if let Some(cudart) = &self.cudart {
                match cudart.bootstrap(device) {
                    Ok(mm) => res.push(mm),
                    Err(e) => {
                        log::debug!("{e}");
                        continue;
                    }
                }
            } else if let Some(nvcuda) = &self.nvcuda {
                match nvcuda.bootstrap(device) {
                    Ok(mm) => res.push(mm),
                    Err(e) => {
                        log::debug!("{e}");
                        continue;
                    }
                }
            };
        }
        res
    }
}

struct CpuHandlers {}

impl CpuHandlers {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
    pub fn get_devices_info(&self) -> Vec<DeviceInfo> {
        let mut cpu = DeviceInfo::default();
        cpu.library = "cpu";
        cpu.variant = CPUCapability::default();
        cpu.memInfo = Self::get_mem();
        vec![cpu]
    }

    fn get_mem() -> MemInfo {
        match cpu::get_mem() {
            Ok(m) => m,
            Err(e) => {
                log::warn!("memory get failed: {e}");
                MemInfo::default()
            }
        }
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
struct MetalHandlers {}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
impl MetalHandlers {
    pub fn new() -> Result<Self> {
        use objc2_metal::MTLDevice;
        let device_name = unsafe {
            let dd = objc2_metal::MTLCreateSystemDefaultDevice();
            (&*dd).name().to_string()
        };
        if device_name == "Apple Paravirtual device" {
            Err(crate::Error::MacParaVirtualDevice)
        } else {
            Ok(Self {})
        }
    }

    pub fn get_devices_info(&self) -> Vec<DeviceInfo> {
        let mut gpu = DeviceInfo::default();
        gpu.library = "metal";
        gpu.id = "0".to_string();
        gpu.minimum_memory = 512 * 1024 * 1024;
        let mm = unsafe {
            iron_oxide::MTLCreateSystemDefaultDevice().get_recommended_max_working_set_size()
        };
        gpu.memInfo = MemInfo {
            total: mm,
            free: mm,
        };
        vec![gpu]
    }
}

#[derive(Debug, Clone)]
struct Variant {
    pub library: String,
    pub variant: String,
}

impl std::fmt::Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.library,
            if self.variant.is_empty() {
                "".to_string()
            } else {
                format!("_{}", self.variant)
            }
        )
    }
}

enum Handlers {
    Cpu(CpuHandlers),
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    Cuda(CudaHandles),
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    Metal(MetalHandlers),
}

impl Handlers {
    pub fn new() -> Result<Self> {
        #[cfg(target_os = "macos")]
        {
            #[cfg(target_arch = "aarch64")]
            if let Ok(h) = MetalHandlers::new() {
                return Ok(Self::Metal(h));
            }
            return Ok(Self::Cpu(CpuHandlers::new()?));
        }
        #[cfg(not(target_os = "macos"))]
        {
            #[cfg(any(target_os = "windows", target_os = "linux"))]
            if let Ok(cuda) = CudaHandles::new() {
                return Ok(Self::Cuda(cuda));
            }
            Ok(Self::Cpu(CpuHandlers::new()?))
        }
    }

    pub fn get_devices_info(&self) -> Vec<DeviceInfo> {
        match self {
            Self::Cpu(h) => h.get_devices_info(),
            #[cfg(any(target_os = "windows", target_os = "linux"))]
            Self::Cuda(h) => h.get_devices_info(),
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            Self::Metal(h) => h.get_devices_info(),
        }
    }

    pub fn available_variants(&self) -> Vec<Variant> {
        let p = DEPENDENCIES_BASE_PATH.display().to_string();
        //can be remove on closing https://github.com/rust-lang/glob/issues/132
        #[cfg(target_os = "windows")]
        let p = if p.starts_with(r###"\\?\"###) {
            &p[4..]
        } else {
            &p[..]
        }
        .to_string();
        match glob::glob(&format!("{}/*/*llama.*", p)) {
            Err(_e) => vec![],
            Ok(entries) => entries.fold(vec![], |mut res, e| {
                if let Ok(path) = e {
                    let v = path
                        .parent()
                        .unwrap()
                        .file_name()
                        .clone()
                        .unwrap()
                        .to_os_string()
                        .into_string();

                    let v = v.unwrap();
                    let mut v = v.split('_');
                    res.push(Variant {
                        library: v.next().unwrap().to_string(),
                        variant: v.next().unwrap_or_default().to_string(),
                    });
                }
                res
            }),
        }
    }

    pub fn llama_cpp(
        &self,
    ) -> Result<(
        libloading::Library,
        libloading::Library,
        libloading::Library,
    )> {
        let devices = self.get_devices_info();
        log::debug!("{devices:#?}");
        let variants = self.available_variants();
        log::debug!("{variants:#?}");
        let mut errs = vec![];
        for device in devices {
            let mut vars = device.variants(&variants);
            vars.sort_by(|a, b| {
                if a.library == "cpu" && b.library == "cpu" {
                    CPUCapability::from(&a.variant).cmp(&CPUCapability::from(&b.variant))
                } else if a.library == "cpu" && b.library != "cpu" {
                    std::cmp::Ordering::Less
                } else if a.library != "cpu" && b.library == "cpu" {
                    std::cmp::Ordering::Greater
                } else if a.library != b.library {
                    unreachable!()
                } else {
                    if a.variant == b.variant {
                        std::cmp::Ordering::Equal
                    } else {
                        let mut a_version =
                            a.variant[1..].split('.').map(|v| v.parse::<i32>().unwrap());
                        let a_v = a_version.next().unwrap_or_default() * 1000
                            + a_version.next().unwrap_or_default();
                        let mut b_version =
                            b.variant[1..].split('.').map(|v| v.parse::<i32>().unwrap());
                        let b_v = b_version.next().unwrap_or_default() * 1000
                            + b_version.next().unwrap_or_default();
                        a_v.cmp(&b_v)
                    }
                }
            });
            vars.reverse();
            log::debug!("{vars:#?}");
            #[cfg(target_os = "windows")]
            {
                let path_env = std::env::var("PATH").unwrap_or_default();
                std::env::set_var(
                    "PATH",
                    path_env
                        + ";"
                        + &DEPENDENCIES_BASE_PATH
                            .clone()
                            .into_os_string()
                            .into_string()
                            .unwrap_or_default(),
                );
            }
            log::debug!(
                "updated PATH: {}",
                std::env::var("PATH").unwrap_or_default()
            );
            for v in vars {
                let mut bp = DEPENDENCIES_BASE_PATH.clone();
                if v.variant.is_empty() {
                    bp.push(v.library.clone());
                } else {
                    bp.push(v.library.clone() + "_" + v.variant.as_str());
                }
                let mut ggml_p = bp.clone();
                #[cfg(target_os = "windows")]
                ggml_p.push("ggml.dll");
                #[cfg(target_os = "macos")]
                ggml_p.push("libggml.dylib");
                #[cfg(target_os = "linux")]
                ggml_p.push("libggml.so");
                let mut llama_p = bp.clone();
                #[cfg(target_os = "windows")]
                llama_p.push("llama.dll");
                #[cfg(target_os = "macos")]
                llama_p.push("libllama.dylib");
                #[cfg(target_os = "linux")]
                llama_p.push("libllama.so");
                let mut llava_p = bp.clone();
                #[cfg(target_os = "windows")]
                llava_p.push("llava_shared.dll");
                #[cfg(target_os = "macos")]
                llava_p.push("libllava_shared.dylib");
                #[cfg(target_os = "linux")]
                llava_p.push("libllava_shared.so");
                match unsafe { libloading::Library::new(ggml_p.clone()) } {
                    Ok(ggml) => match unsafe { libloading::Library::new(llama_p.clone()) } {
                        Ok(llama) => match unsafe { libloading::Library::new(llava_p.clone()) } {
                            Ok(llava) => {
                                log::debug!("variant {v} loaded successfully");
                                return Ok((llama, llava, ggml));
                            }
                            Err(e) => {
                                errs.push(format!("can`t load {}: {}`", llava_p.display(), e));
                                log::warn!("can`t load {}: {}`", llava_p.display(), e);
                                continue;
                            }
                        },
                        Err(e) => {
                            errs.push(format!("can`t load {}: {}`", llama_p.display(), e));
                            log::warn!("can`t load {}: {}`", llama_p.display(), e);
                            continue;
                        }
                    },
                    Err(e) => {
                        errs.push(format!("can`t load {}: {}`", ggml_p.display(), e));
                        log::warn!("can`t load {}: {}`", ggml_p.display(), e);
                        continue;
                    }
                }
            }
        }
        Err(Error::DependenciesLoading(errs))
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn basic_get_gpu_config() {
        let s = super::Handlers::new();
        assert!(s.is_ok());
        let s = s.unwrap();
        assert!(s.len() > 0);
        assert!(["cpu", "cuda", "rocm", "metal"].contains(s[0].library));
        if s[0].library != "cpu" {
            assert!(s[0].memInfo.total > 0);
            assert!(s[0].memInfo.free > 0);
        }
    }
}

struct LlamaCppLibs {
    pub llama_cpp: libloading::Library,
    pub _ggml: libloading::Library,
    pub llava: libloading::Library,
}

#[cfg(target_arch = "x86_64")]
const ARCH: &'static str = "x86_64";
#[cfg(target_arch = "aarch64")]
const ARCH: &'static str = "arm64";

lazy_static::lazy_static! {

    static ref DEPENDENCIES_BASE_PATH: std::path::PathBuf = {
        let mut tt = resource_path::get().unwrap();
        #[cfg(target_os = "windows")]
        tt.push("windows");
        #[cfg(target_os = "linux")]
        tt.push("linux");
        #[cfg(target_os = "macos")]
        tt.push("darwin");

        tt.push(ARCH);
        log::debug!("tmp_dir = {}", tt.display());
        tt
    };

    static ref LIBS: LlamaCppLibs = {
        match Handlers::new(){
            Ok(h) => {
                match h.llama_cpp(){
                    Ok(s) => LlamaCppLibs{
                        llama_cpp: s.0,
                        _ggml: s.2,
                        llava: s.1
                    },
                    Err(e) => panic!("can`t load dependencies: {e}")
                }
            }
            Err(e) => panic!("can`t load dependencies: {e}`")
        }

        //unsafe {libloading::Library::new("libllamacpp.so")}.expect("can`t find lammacpp library")
    };
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    LIbLoading(#[from] libloading::Error),
    #[error("unimplemented: file: {0}, line: {1}")]
    Unimplemented(&'static str, u32),
    #[error("")]
    NvMlLoad,
    #[error("Apple Paravirtual Device")]
    MacParaVirtualDevice,
    #[error("{0}")]
    NvMlInit_v2(i32),
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    #[error("{0}")]
    NvCudaCall(&'static str, i32),
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    #[error("nvcuda load")]
    NvCudaLoad,
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    #[error("{0}")]
    CudartCall(&'static str, i32),
    #[error("{0}")]
    SystemCall(&'static str, i32),
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    #[error("cudart load")]
    CudartLoad,
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    #[error("cuda device not found")]
    CudaNotFound,
    #[cfg(target_os = "linux")]
    #[error("{0}")]
    Proc(#[from] procfs::ProcError),
    #[error("can`t load llama_cpp dependencies {0:#?}")]
    DependenciesLoading(Vec<String>),
}

pub type Result<T> = std::result::Result<T, Error>;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

macro_rules! get_and_load_from_llama
{
    ($($name:tt($($v:ident: $t:ty),* $(,)?) -> $rt:ty),* $(,)?) => {

        $(pub unsafe fn $name($($v: $t),*) -> $rt
        {
            let func: libloading::Symbol<
                unsafe extern "C" fn($($v: $t),*) -> $rt,
                > = LIBS.llama_cpp.get(stringify!($name).as_bytes()).expect(&format!("function \"{}\" not found in llama_cpp lib", stringify!($name)));
            func($($v),*)
        }
        )*
    };
}

macro_rules! get_and_load_from_llava
{
    ($($name:tt($($v:ident: $t:ty),* $(,)?) -> $rt:ty),* $(,)?) => {

        $(pub unsafe fn $name($($v: $t),*) -> $rt
        {
            let func: libloading::Symbol<
                unsafe extern "C" fn($($v: $t),*) -> $rt,
                > = LIBS.llava.get(stringify!($name).as_bytes()).expect(&format!("function \"{}\" not found in llama_cpp lib", stringify!($name)));
            func($($v),*)
        }
        )*
    };
}

get_and_load_from_llava!(
    clip_image_u8_init() -> *mut clip_image_u8,
    clip_image_u8_free(clip: *mut clip_image_u8) -> (),
    clip_image_load_from_bytes(
        image_bytes: *const ::std::os::raw::c_uchar,
        image_bytes_length: ::std::os::raw::c_int,
        image: *mut clip_image_u8
    ) -> bool,
    llava_image_embed_make_with_clip_img(
        clip_context: *const clip_ctx,
        n_threads: i32,
        img: *const clip_image_u8,
        img_embed_out: *mut *mut f32,
        n_img_pos: *mut i32
    ) -> bool,
    clip_model_load(
        fname: *const ::std::os::raw::c_char,
        verbocity: ::std::os::raw::c_int
    ) -> *mut clip_ctx,
    clip_free(ctx: *mut clip_ctx) -> (),
    llava_image_embed_make_with_bytes(
        ctx_clip: *mut clip_ctx,
        n_threads: ::std::os::raw::c_int,
        image_bytes: *const ::std::os::raw::c_uchar,
        image_bytes_length: ::std::os::raw::c_int
    ) -> *mut llava_image_embed,
    llava_image_embed_free(embed: *mut llava_image_embed) -> (),
    llava_eval_image_embed(
        ctx_llama: *mut llama_context,
        embed: *const llava_image_embed,
        n_batch: ::std::os::raw::c_int,
        n_past: *mut ::std::os::raw::c_int
    ) -> bool,
);

get_and_load_from_llama!(
    llama_load_model_from_file(
        path_model: *const ::std::os::raw::c_char,
        params: llama_model_params) -> *mut llama_model,
    llama_tokenize(
        model: *const llama_model,
        text: *const ::std::os::raw::c_char,
        text_len: i32,
        tokens: *mut llama_token,
        n_tokens_max: i32,
        add_special: bool,
        parse_special: bool) -> i32,
    llama_sampler_chain_default_params() -> llama_sampler_chain_params,
    llama_sampler_init_grammar(model: *const llama_model, grammar_str: *const char, grammar_root: *const char) -> *mut llama_sampler,
    llama_sampler_chain_add(chain: *mut llama_sampler, smpl: *const llama_sampler) -> (),
    llama_sampler_init_logit_bias(n_vocab: i32, n_logit_bias: i32, logit_bias: *const llama_logit_bias) -> *mut llama_sampler,
    llama_sampler_init_penalties(n_vocab: i32, special_eos_id: llama_token, linefeed_id: llama_token, penalty_last_n: i32, penalty_repeat: f32, penalty_freq: f32, penalty_present: f32, penalize_nl: bool, ignore_eos: bool) -> *mut llama_sampler,
    llama_model_meta_val_str(model: *const llama_model, key: *const ::std::os::raw::c_char, buf: *mut ::std::os::raw::c_char, bs: usize) -> i32,
    llama_token_is_eog(model: *const llama_model, id: i32) -> bool,
    llama_add_bos_token(model: *const llama_model) -> bool,
    llama_sampler_init_top_k(k: i32) -> *mut llama_sampler,
    llama_sampler_init_top_p(p: f32, min_keep: usize) -> *mut llama_sampler,
    llama_sampler_init_min_p(p: f32, min_keep: usize) -> *mut llama_sampler,
    llama_sampler_init_tail_free(z: f32, min_keep: usize) -> *mut llama_sampler,
    llama_sampler_init_typical(p: f32, min_keep: usize) -> *mut llama_sampler,
    llama_sampler_init_temp_ext(t: f32, delta: f32, exponent: f32) -> *mut llama_sampler,
    llama_sampler_init_softmax() -> *mut llama_sampler,
    llama_sampler_init_dist(seed: u32) -> *mut llama_sampler,
    llama_sampler_init_temp(temp: f32) -> *mut llama_sampler,
    llama_sampler_init_mirostat(n_vocab: i32, seed: u32, tau: f32, eta: f32, m: i32) -> *mut llama_sampler,
    llama_sampler_init_greedy() -> *mut llama_sampler,
    llama_sampler_init_mirostat_v2(seed: u32, tau: f32, eta: f32) -> *mut llama_sampler,
    llama_sampler_chain_init(params: llama_sampler_chain_params) -> *mut llama_sampler,
    llama_sampler_free(smpl: *mut llama_sampler) -> (),
    llama_sampler_accept(smpl: *mut llama_sampler, token: llama_token) -> (),
    llama_sampler_reset(smpl: *mut llama_sampler) -> (),
    llama_get_model(ctx: *const llama_context) -> *mut llama_model,
    llama_sampler_apply(smpl: *mut llama_sampler, cur_p: *const llama_token_data_array) -> (),
    llama_token_get_attr(model: *const llama_model, token: llama_token) -> llama_token_attr,
    llama_token_nl(model: *const llama_model) -> llama_token,
    llama_token_eos(model: *const llama_model) -> llama_token,
    llama_token_bos(model: *const llama_model) -> llama_token,
    llama_n_ctx_train(model: *const llama_model) -> i32,
    llama_free_model(model: *mut llama_model) -> (),
    llama_model_default_params() -> llama_model_params,
    llama_backend_free() -> (),
    llama_log_set(log_callback: ggml_log_callback, user_data: *mut ::std::os::raw::c_void) -> (),
    llama_numa_init(numa: ggml_numa_strategy) -> (),
    llama_backend_init() -> (),
    llama_reset_timings(ctx: *mut llama_context) -> (),
    llama_get_logits_ith(ctx: *mut llama_context, i: i32) -> *mut f32,
    llama_get_embeddings_ith(ctx: *mut llama_context, i: i32) -> *mut f32,
    llama_get_embeddings_seq(ctx: *mut llama_context, seq_id: llama_seq_id) -> *mut f32,
    llama_decode(ctx: *mut llama_context, batch: llama_batch) -> i32,
    llama_n_ctx(ctx: *const llama_context) -> u32,
    llama_n_batch(ctx: *const llama_context) -> u32,
    llama_free(ctx: *mut llama_context) -> (),
    llama_set_state_data(ctx: *mut llama_context, src: *const u8) -> usize,
    llama_copy_state_data(ctx: *mut llama_context, dst: *mut u8) -> usize,
    llama_get_state_size(ctx: *const llama_context) -> usize,
    llama_load_session_file(
        ctx: *mut llama_context,
        path_session: *const ::std::os::raw::c_char,
        tokens_out: *mut llama_token,
        n_token_capacity: usize,
        n_token_count_out: *mut usize
    ) -> bool,
    llama_save_session_file(
        ctx: *mut llama_context,
        path_session: *const ::std::os::raw::c_char,
        tokens: *const llama_token,
        n_token_count: usize
    ) -> bool,
    llama_sample_token_greedy(
        ctx: *mut llama_context,
        candidates: *mut llama_token_data_array
    ) -> llama_token,
    llama_context_default_params() -> llama_context_params,
    llama_kv_cache_view_free(view: *mut llama_kv_cache_view) -> (),
    llama_kv_cache_view_update(ctx: *const llama_context, view: *mut llama_kv_cache_view) -> (),
    llama_kv_cache_view_init(
        ctx: *const llama_context,
        n_seq_max: i32
    ) -> llama_kv_cache_view,
    llama_get_kv_cache_token_count(ctx: *const llama_context) -> i32,
    llama_kv_cache_update(ctx: *mut llama_context) -> (),
    llama_kv_cache_defrag(ctx: *mut llama_context) -> (),
    llama_kv_cache_seq_pos_max(ctx: *mut llama_context, seq_id: llama_seq_id) -> llama_pos,
    llama_kv_cache_seq_div(
        ctx: *mut llama_context,
        seq_id: llama_seq_id,
        p0: llama_pos,
        p1: llama_pos,
        d: ::std::os::raw::c_int
    ) -> (),
    llama_kv_cache_seq_add(
        ctx: *mut llama_context,
        seq_id: llama_seq_id,
        p0: llama_pos,
        p1: llama_pos,
        delta: llama_pos
    ) -> (),
    llama_kv_cache_seq_keep(ctx: *mut llama_context, seq_id: llama_seq_id) -> (),
    llama_kv_cache_clear(ctx: *mut llama_context) -> (),
    llama_get_kv_cache_used_cells(ctx: *const llama_context) -> i32,
    llama_kv_cache_seq_rm(
        ctx: *mut llama_context,
        seq_id: llama_seq_id,
        p0: llama_pos,
        p1: llama_pos
    ) -> bool,
    llama_kv_cache_seq_cp(
        ctx: *mut llama_context,
        seq_id_src: llama_seq_id,
        seq_id_dst: llama_seq_id,
        p0: llama_pos,
        p1: llama_pos
    ) -> (),
    llama_supports_mlock() -> bool,
    llama_supports_mmap() -> bool,
    llama_max_devices() -> usize,
    llama_sample_token_mirostat_v2(
        ctx: *mut llama_context,
        candidates: *mut llama_token_data_array,
        tau: f32,
        eta: f32,
        mu: *mut f32
    ) -> llama_token,
    llama_sample_min_p(
        ctx: *mut llama_context,
        candidates: *mut llama_token_data_array,
        p: f32,
        min_keep: usize,
    ) -> (),
    llama_sample_top_p(
        ctx: *mut llama_context,
        candidates: *mut llama_token_data_array,
        p: f32,
        min_keep: usize,
    ) -> (),
    llama_sample_top_k(
        ctx: *mut llama_context,
        candidates: *mut llama_token_data_array,
        p: i32,
        min_keep: usize,
    ) -> (),
    llama_sample_typical(
        ctx: *mut llama_context,
        candidates: *mut llama_token_data_array,
        p: f32,
        min_keep: usize,
    ) -> (),
    llama_sample_tail_free(
        ctx: *mut llama_context,
        candidates: *mut llama_token_data_array,
        p: f32,
        min_keep: usize,
    ) -> (),
    llama_sample_token(
        ctx: *mut llama_context,
        candidates: *mut llama_token_data_array,
    ) -> llama_token,
    llama_sample_temp(
        ctx: *mut llama_context,
        candidates: *mut llama_token_data_array,
        temp: f32,
    ) -> (),
    llama_sample_softmax(
        ctx: *mut llama_context,
        candidates: *mut llama_token_data_array,
    ) -> (),
    llama_sample_repetition_penalties(
        ctx: *mut llama_context,
        candidates: *mut llama_token_data_array,
        last_tokens: *const llama_token,
        penalty_last_n: usize,
        penalty_repeat: f32,
        penalty_freq: f32,
        penalty_present: f32,
    ) -> (),
    llama_new_context_with_model(
        model: *mut llama_model,
        params: llama_context_params,
    ) -> *mut llama_context,
    llama_n_embd(model: *const llama_model) -> i32,
    llama_n_vocab(model: *const llama_model) -> i32,
    llama_vocab_type(model: *const llama_model) -> llama_vocab_type,
    llama_token_to_piece(
        model: *const llama_model,
        token: llama_token,
        buf: *mut ::std::os::raw::c_char,
        length: i32,
        special: bool,
    ) -> i32,
    llama_time_us() -> i64,
    ggml_time_us() -> i64,
    llama_batch_init(n_tokens: i32, embd: i32, n_seq_max: i32) -> llama_batch,
    llama_batch_free(batch: llama_batch) -> ()
);
