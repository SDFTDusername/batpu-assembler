#[derive(Debug, Copy, Clone)]
pub struct AssemblerConfig {
    pub default_defines: bool,
    pub print_info: bool,
    pub text_output: bool
}

impl Default for AssemblerConfig {
    fn default() -> Self {
        Self {
            default_defines: true,
            print_info: false,
            text_output: false
        }
    }
}