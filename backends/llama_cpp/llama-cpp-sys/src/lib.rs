#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::fmt::{Debug, Formatter};

mod cuda;

struct CudaHandles {
    device_count: usize,
    cudart: Option<cuda::cudart::CudartHandle>,
    nvcuda: Option<cuda::nvcuda::NvCudaHandle>,
    nvml: Option<cuda::nvml::NvMlHandle>,
    pub llamacpp: libloading::Library,
    llava: libloading::Library,
}

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
        let (device_count, cudart) = match cuda::cudart::CudartHandle::new() {
            Ok((d, h)) => (d, Some(h)),
            Err(e) => {
                log::warn!("{e}");
                (0, None)
            }
        };
        Ok(Self {
            nvml,
            device_count,
            nvcuda,
            cudart,
            llamacpp: unsafe { libloading::Library::new("libllamacpp.so")? },
            llava: unsafe { libloading::Library::new("libllamacpp.so")? },
        })
    }
}

struct CpuHandlers {
    pub llamacpp: libloading::Library,
    llava: libloading::Library,
}

impl CpuHandlers {
    pub fn new() -> Result<Self> {
        Ok(if ::std::is_x86_feature_detected!("avx2") {
            Self {
                llamacpp: unsafe { libloading::Library::new("libllamacpp.so")? },
                llava: unsafe { libloading::Library::new("libllamacpp.so")? },
            }
        } else if ::std::is_x86_feature_detected!("avx") {
            Self {
                llamacpp: unsafe { libloading::Library::new("libllamacpp.so")? },
                llava: unsafe { libloading::Library::new("libllamacpp.so")? },
            }
        } else {
            Self {
                llamacpp: unsafe { libloading::Library::new("libllamacpp.so")? },
                llava: unsafe { libloading::Library::new("libllamacpp.so")? },
            }
        })
    }
}

enum Handlers {
    Cpu(CpuHandlers),
    Cuda(CudaHandles),
}

impl Handlers {
    pub fn new() -> Result<Self> {
        if let Ok(cuda) = CudaHandles::new() {
            Ok(Self::Cuda(cuda))
        } else {
            Ok(Self::Cpu(CpuHandlers::new()?))
        }
    }

    pub fn llama_cpp(&self) -> &libloading::Library {
        match self {
            Self::Cpu(h) => &h.llamacpp,
            Self::Cuda(h) => &h.llamacpp,
        }
    }
}

lazy_static::lazy_static! {
    static ref LIBS: Handlers = {
        match Handlers::new(){
            Ok(h) => h,
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
    #[error("{0}")]
    NvMlInit_v2(i32),
}

pub type Result<T> = std::result::Result<T, Error>;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

macro_rules! get_and_load
{
    ($($name:tt($($v:ident: $t:ty),* $(,)?) -> $rt:ty),* $(,)?) => {

        $(pub unsafe fn $name($($v: $t),*) -> $rt
        {
            let func: libloading::Symbol<
                unsafe extern "C" fn($($v: $t),*) -> $rt,
                > = LIBS.llama_cpp().get(stringify!($name).as_bytes()).expect(&format!("function \"{}\" not found in llama_cpp lib", stringify!($name)));
            func($($v),*)
        }
        )*
    };
}

get_and_load!(
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
    llama_token_get_type(model: *const llama_model, token: llama_token) -> llama_token_type,
    llama_token_nl(model: *const llama_model) -> llama_token,
    llama_token_eos(model: *const llama_model) -> llama_token,
    llama_token_bos(model: *const llama_model) -> llama_token,
    llama_n_ctx_train(model: *const llama_model) -> i32,
    llama_free_model(model: *mut llama_model) -> (),
    llama_model_default_params() -> llama_model_params,
    llama_backend_free() -> (),
    llama_log_set(log_callback: ggml_log_callback, user_data: *mut ::std::os::raw::c_void) -> (),
    llama_numa_init(numa: ggml_numa_strategy) -> (),
    llama_grammar_free(grammar: *mut llama_grammar) -> (),
    llama_backend_init() -> (),
    llama_grammar_init(
        rules: *mut *const llama_grammar_element,
        n_rules: usize,
        start_rule_index: usize
    ) -> *mut llama_grammar,
    llama_grammar_copy(grammar: *const llama_grammar) -> *mut llama_grammar,
    llava_eval_image_embed(
        ctx_llama: *mut llama_context,
        embed: *const llava_image_embed,
        n_batch: ::std::os::raw::c_int,
        n_past: *mut ::std::os::raw::c_int
    ) -> bool,
    llama_get_timings(ctx: *mut llama_context) -> llama_timings,
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
    llama_sample_grammar(
        ctx: *mut llama_context,
        candidates: *mut llama_token_data_array,
        grammar: *const llama_grammar
    ) -> (),
    llama_grammar_accept_token(
        ctx: *mut llama_context,
        grammar: *mut llama_grammar,
        token: llama_token
    ) -> (),
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
    clip_free(ctx: *mut clip_ctx) -> (),
    llava_image_embed_make_with_bytes(
        ctx_clip: *mut clip_ctx,
        n_threads: ::std::os::raw::c_int,
        image_bytes: *const ::std::os::raw::c_uchar,
        image_bytes_length: ::std::os::raw::c_int
    ) -> *mut llava_image_embed,
    llava_image_embed_free(embed: *mut llava_image_embed) -> (),
    clip_model_load(
        fname: *const ::std::os::raw::c_char,
        verbocity: ::std::os::raw::c_int
    ) -> *mut clip_ctx,
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

impl Debug for llama_grammar_element {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fn type_to_str(r#type: llama_gretype) -> &'static str {
            match r#type {
                LLAMA_GRETYPE_END => "END",
                LLAMA_GRETYPE_ALT => "ALT",
                LLAMA_GRETYPE_RULE_REF => "RULE_REF",
                LLAMA_GRETYPE_CHAR => "CHAR",
                LLAMA_GRETYPE_CHAR_NOT => "CHAR_NOT",
                LLAMA_GRETYPE_CHAR_RNG_UPPER => "CHAR_RNG_UPPER",
                LLAMA_GRETYPE_CHAR_ALT => "CHAR_ALT",
                _ => "Unknown",
            }
        }

        f.debug_struct("llama_grammar_element")
            .field("type", &type_to_str(self.type_))
            .field("value", &self.value)
            .finish()
    }
}
