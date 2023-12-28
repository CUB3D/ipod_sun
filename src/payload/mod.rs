pub mod code_exec;
pub mod exploit_config;
pub mod payload_builder;
//mod binary_search;
//mod string_leak;

use crate::payload::exploit_config::ExploitConfig;
use crate::payload::payload_builder::CffPayloadBuilder;

pub trait Payload {
    fn build_cff<Cfg: ExploitConfig>(&self, builder: &mut CffPayloadBuilder);
}
