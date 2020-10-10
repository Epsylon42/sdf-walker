//! The [glutin] windowing implementation for [luminance-windowing].
//!
//! [glutin]: https://crates.io/crates/glutin
//! [luminance-windowing]: https://crates.io/crates/luminance-windowing

#![deny(missing_docs)]

use gl;
use glutin::{
    event_loop::EventLoop, Api, ContextBuilder, ContextError, CreationError,
    GlProfile, GlRequest, NotCurrent, PossiblyCurrent, Context,
};
use luminance::context::GraphicsContext;
use luminance::framebuffer::{Framebuffer, FramebufferError};
use luminance::texture::Dim2;
pub use luminance_gl::gl33::StateQueryError;
use luminance_gl::GL33;
use std::error;
use std::fmt;
use std::os::raw::c_void;
use glutin::platform::unix::HeadlessContextExt;

/// Error that might occur when creating a Glutin surface.
#[derive(Debug)]
pub enum GlutinError {
    /// Something went wrong when creating the Glutin surface. The carried [`CreationError`] provides
    /// more information.
    CreationError(CreationError),
    /// OpenGL context error.
    ContextError(ContextError),
    /// Graphics state error that might occur when querying the initial state.
    GraphicsStateError(StateQueryError),
}

impl fmt::Display for GlutinError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            GlutinError::CreationError(ref e) => write!(f, "Glutin surface creation error: {}", e),
            GlutinError::ContextError(ref e) => write!(f, "Glutin OpenGL context creation error: {}", e),
            GlutinError::GraphicsStateError(ref e) => {
                write!(f, "OpenGL graphics state initialization error: {}", e)
            }
        }
    }
}

impl error::Error for GlutinError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            GlutinError::CreationError(e) => Some(e),
            GlutinError::ContextError(e) => Some(e),
            GlutinError::GraphicsStateError(e) => Some(e),
        }
    }
}

impl From<CreationError> for GlutinError {
    fn from(e: CreationError) -> Self {
        GlutinError::CreationError(e)
    }
}

impl From<ContextError> for GlutinError {
    fn from(e: ContextError) -> Self {
        GlutinError::ContextError(e)
    }
}

impl From<StateQueryError> for GlutinError {
    fn from(e: StateQueryError) -> Self {
        GlutinError::GraphicsStateError(e)
    }
}

/// The Glutin surface.
///
/// You want to create such an object in order to use any [luminance] construct.
///
/// [luminance]: https://crates.io/crates/luminance
pub struct GlutinOffscreen {
    /// The windowed context.
    pub ctx: Context<PossiblyCurrent>,
    /// OpenGL 3.3 state.
    gl: GL33,
}

unsafe impl GraphicsContext for GlutinOffscreen {
    type Backend = GL33;

    fn backend(&mut self) -> &mut Self::Backend {
        &mut self.gl
    }
}

impl GlutinOffscreen {
    /// Create a new [`GlutinOffscreen`]
    ///
    pub fn new_gl33_from_builder<'a>(
        el: &EventLoop<()>,
        ctx_builder: ContextBuilder<'a, NotCurrent>,
    ) -> Result<Self, GlutinError>
    {
        let ctx = ctx_builder.build_headless(el, (1, 1).into())?;

        let ctx = unsafe { ctx.make_current().map_err(|(_, e)| e)? };

        // init OpenGL
        gl::load_with(|s| ctx.get_proc_address(s) as *const c_void);

        let gl = GL33::new().map_err(GlutinError::GraphicsStateError)?;
        let surface = GlutinOffscreen { ctx, gl };

        Ok(surface)
    }
}
