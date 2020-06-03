//! An internal library to manage VFPU register contexts.
//!
//! This is similar (but not identical) to the pspvfpu library from PSPSDK.

use crate::sys::types::{ScePspFMatrix4, ScePspFVector4};

const NUM_MATRICES: usize = 8;

bitflags::bitflags! {
    pub struct MatrixSet: u8 {
        const VMAT0 = 0b0000_0001;
        const VMAT1 = 0b0000_0010;
        const VMAT2 = 0b0000_0100;
        const VMAT3 = 0b0000_1000;
        const VMAT4 = 0b0001_0000;
        const VMAT5 = 0b0010_0000;
        const VMAT6 = 0b0100_0000;
        const VMAT7 = 0b1000_0000;
    }
}

#[repr(C, align(16))]
pub struct Context {
    matrices: [ScePspFMatrix4; NUM_MATRICES],
    saved: MatrixSet,
}

impl Context {
    pub fn new() -> Self {
        unsafe {
            use crate::sys::kernel::{self, ThreadAttributes};

            // TODO: Handle errors.
            kernel::sce_kernel_change_current_thread_attr(0, ThreadAttributes::VFPU);
        }

        let zero_vector = ScePspFVector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
        let zero_matrix = ScePspFMatrix4 {
            x: zero_vector,
            y: zero_vector,
            z: zero_vector,
            w: zero_vector,
        };

        let matrices = [
            zero_matrix,
            zero_matrix,
            zero_matrix,
            zero_matrix,

            zero_matrix,
            zero_matrix,
            zero_matrix,
            zero_matrix,
        ];

        Self {
            matrices,
            saved: MatrixSet::empty(),
        }
    }

    fn restore(&mut self, matrix_idx: u8) {
        macro_rules! restore {
            ($restore_addr:expr, $c0:ident, $c1:ident, $c2:ident, $c3:ident) => {
                vfpu_asm! {
                    lv_q $c0, t0;
                    lv_q $c1, 16(t0);
                    lv_q $c2, 32(t0);
                    lv_q $c3, 48(t0);

                    : : "{t0}"($restore_addr) : "memory" : "volatile"
                }
            }
        }

        let idx = matrix_idx as usize;

        unsafe {
            match matrix_idx {
                0 => restore!(&self.matrices[idx], C000, C010, C020, C030),
                1 => restore!(&self.matrices[idx], C100, C110, C120, C130),
                2 => restore!(&self.matrices[idx], C200, C210, C220, C230),
                3 => restore!(&self.matrices[idx], C300, C310, C320, C330),
                4 => restore!(&self.matrices[idx], C400, C410, C420, C430),
                5 => restore!(&self.matrices[idx], C500, C510, C520, C530),
                6 => restore!(&self.matrices[idx], C600, C610, C620, C630),
                7 => restore!(&self.matrices[idx], C700, C710, C720, C730),
                _ => core::intrinsics::unreachable(),
            }

            self.saved &= !MatrixSet::from_bits_unchecked(1 << matrix_idx);
        }
    }

    fn save(&mut self, matrix_idx: u8) {
        macro_rules! save {
            ($save_addr:expr, $c0:ident, $c1:ident, $c2:ident, $c3:ident) => {
                vfpu_asm! {
                    sv_q $c0, t0;
                    sv_q $c1, 16(t0);
                    sv_q $c2, 32(t0);
                    sv_q $c3, 48(t0);

                    : : "{t0}"($save_addr) : "memory" : "volatile"
                }
            }
        }

        let idx = matrix_idx as usize;

        unsafe {
            match matrix_idx {
                0 => save!(&mut self.matrices[idx], C000, C010, C020, C030),
                1 => save!(&mut self.matrices[idx], C100, C110, C120, C130),
                2 => save!(&mut self.matrices[idx], C200, C210, C220, C230),
                3 => save!(&mut self.matrices[idx], C300, C310, C320, C330),
                4 => save!(&mut self.matrices[idx], C400, C410, C420, C430),
                5 => save!(&mut self.matrices[idx], C500, C510, C520, C530),
                6 => save!(&mut self.matrices[idx], C600, C610, C620, C630),
                7 => save!(&mut self.matrices[idx], C700, C710, C720, C730),
                _ => core::intrinsics::unreachable(),
            }

            self.saved |= MatrixSet::from_bits_unchecked(1 << matrix_idx);
        }
    }

    pub unsafe fn prepare(&mut self, in_out: MatrixSet, clobber: MatrixSet) {
        for i in 0..8 {
            let matrix = MatrixSet::from_bits_unchecked(1 << i);

            if in_out.intersects(matrix) && self.saved.intersects(matrix) {
                self.restore(i);
            } else if clobber.intersects(matrix) && !self.saved.intersects(matrix) {
                self.save(i);
            }
        }
    }
}
