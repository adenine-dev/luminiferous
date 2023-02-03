use glam::*;

pub trait Integrate {
    fn render_fragment(&self, fragment: IVec2) -> Vec4;
}

mod simple;
pub use simple::*;

// HACK: this stuff is to get around not being able to use adts in rustgpu rn, that's supposed to be lifted soon tho

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
enum IntegratorType {
    Simple,
}

#[repr(C)]
pub struct Integrator {
    _pad: u32,
    integrator_type: IntegratorType,
    simple: SimpleIntegrator,
}

impl Integrator {
    pub fn simple_integrator(size: Vec2) -> Self {
        Self {
            integrator_type: IntegratorType::Simple,
            simple: SimpleIntegrator::new(size),
            _pad: 0,
        }
    }
}

impl Integrate for Integrator {
    fn render_fragment(&self, fragment: IVec2) -> Vec4 {
        if self.integrator_type == IntegratorType::Simple {
            self.simple.render_fragment(fragment)
        } else {
            unimplemented!()
        }
    }
}

// pub enum Integrator {
//     SimpleIntegrator(SimpleIntegrator),
// }

// impl Integrator {
//     pub fn simple_integrator(color: Vec4) -> Self {
//         Self::SimpleIntegrator(SimpleIntegrator::new(color))
//     }
// }

// impl Integrate for Integrator {
//     fn render_fragment(&self, fragment: IVec2) -> Vec4 {
//         match self {
//             Self::SimpleIntegrator(s) => s.render_fragment(fragment),
//             _ => panic!("oof"), // Self::Other(_) => panic!("oof"),
//         }
//     }
// }
