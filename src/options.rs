fn default_i32_minus1() -> i32 {
    -1
}

fn default_usize_2048() -> usize {
    2048
}

fn default_n_threads() -> usize {
    num_cpus::get()
}

#[derive(serde::Deserialize)]
pub struct ModelOptions {
    #[serde(default)]
    pub cpu: bool,
    #[serde(default = "default_i32_minus1")]
    pub n_gpu_layers: i32,
}

impl ModelOptions {
    pub fn use_cpu(mut self) -> Self {
        self.cpu = true;
        self
    }

    pub fn with_n_gpu_layers(mut self, n_gpu_layers: i32) -> Self {
        self.n_gpu_layers = n_gpu_layers;
        self
    }
}

impl Default for ModelOptions {
    fn default() -> Self {
        Self {
            cpu: false,
            n_gpu_layers: default_i32_minus1(),
        }
    }
}

fn default_conversation_user_format() -> String {
    "User:\n{prompt}\n".to_string()
}

fn default_conversation_assistant_format() -> String {
    "Assistant:\n{prompt}\n".to_string()
}

fn default_conversation_prompt_format() -> String {
    "User:\n{prompt}\nAssistant:\n".to_string()
}

fn default_conversation_prompt_format_with_image() -> String {
    "{image}User:\n{prompt}\nAssistant:\n".to_string()
}

fn default_stop_tokens() -> Vec<String> {
    vec!["User:", "Assistant:"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

#[derive(Clone, serde::Deserialize)]
pub struct Message {
    pub message: String,
    pub is_user: bool,
}

#[derive(Clone, serde::Deserialize)]
pub struct ContextOptions {
    #[serde(default)]
    pub seed: u32,
    #[serde(default = "default_usize_2048")]
    pub n_ctx: usize,
    #[serde(default = "default_n_threads")]
    pub n_threads: usize,
    #[serde(default = "default_conversation_user_format")]
    pub user_format: String,
    #[serde(default = "default_conversation_assistant_format")]
    pub assistant_format: String,
    #[serde(default = "default_conversation_prompt_format")]
    pub prompt_format: String,
    #[serde(default = "default_conversation_prompt_format_with_image")]
    pub prompt_format_with_image: String,
    #[serde(default = "default_stop_tokens")]
    pub stop_tokens: Vec<String>,
    #[serde(default)]
    pub ctx: Vec<Message>,
}

impl ContextOptions {
    pub fn with_seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }

    pub fn with_n_ctx(mut self, n_ctx: usize) -> Self {
        self.n_ctx = n_ctx;
        self
    }

    pub fn with_conversation_user_format(mut self, format: &str) -> Self {
        self.user_format = format.into();
        self
    }

    pub fn with_conversation_assistant_format(mut self, format: &str) -> Self {
        self.assistant_format = format.into();
        self
    }

    pub fn with_conversation_prompt_format(mut self, format: &str) -> Self {
        self.prompt_format = format.into();
        self
    }

    pub fn with_conversation_prompt_format_with_image(mut self, format: &str) -> Self {
        self.prompt_format_with_image = format.into();
        self
    }

    pub fn with_stop_tokens(mut self, tokens: &[&str]) -> Self {
        self.stop_tokens = tokens.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_ctx(mut self, messages: Vec<Message>) -> Self {
        self.ctx = messages;
        self
    }
}

impl Default for ContextOptions {
    fn default() -> Self {
        Self {
            seed: 0,
            n_ctx: default_usize_2048(),
            n_threads: default_n_threads(),
            user_format: default_conversation_user_format(),
            assistant_format: default_conversation_assistant_format(),
            prompt_format: default_conversation_prompt_format(),
            prompt_format_with_image: default_conversation_prompt_format_with_image(),
            stop_tokens: default_stop_tokens(),
            ctx: vec![],
        }
    }
}

pub struct NebulaOptions {
    pub seed: i32,
    pub n_threads: i32,
    pub n_threads_draft: i32,
    pub n_threads_batch: i32,
    pub n_threads_batch_draft: i32,
    pub n_predict: i32,
    pub n_ctx: i32,
    pub n_batch: i32,
    pub n_ubatch: i32,
    pub n_keep: i32,
    pub n_draft: i32,
    pub n_chunks: i32,
    pub n_parallel: i32,
    pub n_sequences: i32,
    pub p_split: f32,
    pub n_gpu_layers: i32,
    pub n_gpu_layers_draft: i32,
    //pub split_mode: SplitMode,
    pub main_gpu: i32,
    pub tensor_split: [f32; 128],
    pub n_beams: i32,
    pub grp_attn_n: i32,
    pub grp_attn_w: i32,
    pub n_print: i32,
    pub rope_freq_base: f32,
    pub rope_freq_scale: f32,
    pub yarn_ext_factor: f32,
    pub yarn_attn_factor: f32,
    pub yarn_beta_fast: f32,
    pub yarn_beta_slow: f32,
    pub yarn_orig_ctx: i32,
    pub defrag_thold: f32,
    //pub numa: NumaStrategy,
    //pub rope_scaling_type: RopeScalingType,
    //pub pooling_type: PoolingType,
    //pub sparams: SamplingParams;
}

impl NebulaOptions {
    pub fn with_seed(mut self, val: i32) -> Self {
        self.seed = val;
        self
    }
    pub fn with_n_threads(mut self, val: i32) -> Self {
        self.n_threads = val;
        self
    }
    pub fn with_n_threads_draft(mut self, val: i32) -> Self {
        self.n_threads_draft = val;
        self
    }
    pub fn with_n_threads_batch(mut self, val: i32) -> Self {
        self.n_threads_batch = val;
        self
    }
    pub fn with_n_threads_batch_draft(mut self, val: i32) -> Self {
        self.n_threads_batch_draft = val;
        self
    }
    pub fn with_n_predict(mut self, val: i32) -> Self {
        self.n_predict = val;
        self
    }
    pub fn with_n_ctx(mut self, val: i32) -> Self {
        self.n_ctx = val;
        self
    }
    pub fn with_n_batch(mut self, val: i32) -> Self {
        self.n_batch = val;
        self
    }
    pub fn with_n_ubatch(mut self, val: i32) -> Self {
        self.n_ubatch = val;
        self
    }
    pub fn with_n_keep(mut self, val: i32) -> Self {
        self.n_keep = val;
        self
    }
    pub fn with_n_draft(mut self, val: i32) -> Self {
        self.n_draft = val;
        self
    }
    pub fn with_n_chunks(mut self, val: i32) -> Self {
        self.n_chunks = val;
        self
    }
    pub fn with_n_parallel(mut self, val: i32) -> Self {
        self.n_parallel = val;
        self
    }
    pub fn with_n_sequences(mut self, val: i32) -> Self {
        self.n_sequences = val;
        self
    }
    pub fn with_p_split(mut self, val: f32) -> Self {
        self.p_split = val;
        self
    }
    pub fn with_n_gpu_layers(mut self, val: i32) -> Self {
        self.n_gpu_layers = val;
        self
    }
    pub fn with_n_gpu_layers_draft(mut self, val: i32) -> Self {
        self.n_gpu_layers_draft = val;
        self
    }
    //pub split_mode: SplitMode,
    pub fn with_main_gpu(mut self, val: i32) -> Self {
        self.main_gpu = val;
        self
    }
    pub fn with_tensor_split(mut self, val: [f32; 128]) -> Self {
        self.tensor_split = val;
        self
    }
    pub fn with_n_beams(mut self, val: i32) -> Self {
        self.n_beams = val;
        self
    }
    pub fn with_grp_attn_n(mut self, val: i32) -> Self {
        self.grp_attn_n = val;
        self
    }
    pub fn with_grp_attn_w(mut self, val: i32) -> Self {
        self.grp_attn_w = val;
        self
    }
    pub fn with_n_print(mut self, val: i32) -> Self {
        self.n_print = val;
        self
    }
    pub fn with_rope_freq_base(mut self, val: f32) -> Self {
        self.rope_freq_base = val;
        self
    }
    pub fn with_rope_freq_scale(mut self, val: f32) -> Self {
        self.rope_freq_scale = val;
        self
    }
    pub fn with_yarn_ext_factor(mut self, val: f32) -> Self {
        self.yarn_ext_factor = val;
        self
    }
    pub fn with_yarn_attn_factor(mut self, val: f32) -> Self {
        self.yarn_attn_factor = val;
        self
    }
    pub fn with_yarn_beta_fast(mut self, val: f32) -> Self {
        self.yarn_beta_fast = val;
        self
    }
    pub fn with_yarn_beta_slow(mut self, val: f32) -> Self {
        self.yarn_beta_slow = val;
        self
    }
    pub fn with_yarn_orig_ctx(mut self, val: i32) -> Self {
        self.yarn_orig_ctx = val;
        self
    }
    pub fn with_defrag_thold(mut self, val: f32) -> Self {
        self.defrag_thold = val;
        self
    }
    //pub numa: NumaStrategy,
    //pub rope_scaling_type: RopeScalingType,
    //pub pooling_type: PoolingType,
    //pub sparams: SamplingParams;
}

impl Default for NebulaOptions {
    fn default() -> Self {
        Self {
            seed: -1,
            n_threads: num_cpus::get() as i32,
            n_threads_draft: -1,
            n_threads_batch: -1,
            n_threads_batch_draft: -1,
            n_predict: -1,
            n_ctx: 512,
            n_batch: 2048,
            n_ubatch: 512,
            n_keep: 0,
            n_draft: 5,
            n_chunks: -1,
            n_parallel: 1,
            n_sequences: 1,
            p_split: 0.1,
            n_gpu_layers: -1,
            n_gpu_layers_draft: -1,
            //split_mode: SplitMode::Layer,
            main_gpu: 0,
            tensor_split: [0.0; 128],
            n_beams: 0,
            grp_attn_n: 1,
            grp_attn_w: 512,
            n_print: -1,
            rope_freq_base: 0.0,
            rope_freq_scale: 0.0,
            yarn_ext_factor: -1.0,
            yarn_attn_factor: 1.0,
            yarn_beta_fast: 32.0,
            yarn_beta_slow: 1.0,
            yarn_orig_ctx: 0,
            defrag_thold: 1.0,
            //numa: NumaStrategy::Disabled,
            //rope_scaling_type: RopeScalingType::Unspecified,
            //pooling_type: PoolingType::Unspecified,
            //sparams: SamplingParams::default()
        }
    }
}
