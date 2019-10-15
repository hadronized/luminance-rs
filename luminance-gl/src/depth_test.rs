use gl::types::*;
use luminance::depth_test::DepthComparison;

pub(crate) fn depth_comparison_to_glenum(comp: DepthComparison) -> GLenum {
  match comp {
    DepthComparison::Never => gl::NEVER,
    DepthComparison::Always => gl::ALWAYS,
    DepthComparison::Equal => gl::EQUAL,
    DepthComparison::NotEqual => gl::NOTEQUAL,
    DepthComparison::Less => gl::LESS,
    DepthComparison::LessOrEqual => gl::LEQUAL,
    DepthComparison::Greater => gl::GREATER,
    DepthComparison::GreaterOrEqual => gl::GEQUAL,
  }
}
