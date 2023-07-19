// adapted from https://github.com/rgl-epfl/brdf-loader

use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::Path,
    usize,
};

use crate::prelude::*;
use crate::spectra::SpectrumT;
use crate::{primitive::SurfaceInteraction, spectra::Spectrum, stats::STATS};

use super::{reflect_across, spherical_theta, BsdfFlags, BsdfSample, BsdfT};

pub(crate) fn find_interval<F>(sized: usize, mut f: F) -> usize
where
    F: FnMut(usize) -> bool,
{
    let mut size = sized as isize - 2;
    let mut first = 1;
    while size > 0 {
        let half = size >> 1;
        let middle = first + half;
        let pr = f(middle as usize);

        first = if pr { middle + 1 } else { first };
        size = if pr { size - (half + 1) } else { half };
    }

    (first - 1).clamp(0, (sized as isize - 2).max(0)) as usize
}

#[derive(Debug, Clone)]
pub struct Marginal2d<const N: usize> {
    /// Resolution of the discretized density function
    size: UVector2,

    /// Size of a bilinear patch in the unit square
    patch_size: Vector2,
    inv_patch_size: Vector2,

    /// Resolution of each parameter (optional)
    param_size: [usize; N],

    /// Stride per parameter in units of sizeof(float)
    param_strides: [usize; N],

    /// Discretization of each parameter domain
    param_values: [Vec<f32>; N],

    /// Density values
    data: Vec<f32>,

    /// Marginal and conditional PDFs
    marginal_cdf: Vec<f32>,
    conditional_cdf: Vec<f32>,
}

impl<const N: usize> Marginal2d<N>
where
    [(); 2 * N]:,
{
    pub fn new(
        size: UVector2,
        data: &[f32],
        param_values: [Vec<f32>; N],
        normalize: bool,
        build_cdf: bool,
    ) -> Result<Self, Box<dyn Error>> {
        if build_cdf && !normalize {
            return Err("if `build_cdf` is true, `normalize` must also be true")?;
        }
        let mut slices = 1usize;
        let mut param_size = [0; N];
        let mut param_strides = [0; N];
        for i in (0..N).rev() {
            // FIXME: was reversed before
            if param_values[i].is_empty() {
                return Err("parameter resolution not be empty")?;
            }
            param_size[i] = param_values[i].len();
            param_strides[i] = if param_size[i] > 1 { slices } else { 0 };
            slices *= param_size[i];
        }

        let inv_patch_size = (size - UVector2::splat(1)).as_vec2();

        let n_values = (size.x * size.y) as usize;
        let mut data_out = vec![0.0; n_values * slices];
        let mut data_out_off = 0;
        let mut data_off = 0;

        let (marginal_cdf, conditional_cdf) = if build_cdf {
            let mut marginal_cdf = vec![0.0; slices * size.y as usize];
            let mut conditional_cdf = vec![0.0; slices * n_values];

            let mut marginal_off = 0;
            let mut conditional_off = 0;

            for _slice in 0..slices {
                for y in 0..size.y as usize {
                    let mut sum = 0.0;
                    let mut i = y * size.x as usize;
                    for _x in 0..(size.x as usize - 1) {
                        sum += 0.5 * (data[data_off + i] as f64 + data[data_off + i + 1] as f64);
                        conditional_cdf[conditional_off + i + 1] = sum as f32;
                        i += 1;
                    }
                }

                marginal_cdf[marginal_off] = 0.0;
                let mut sum = 0.0;
                for y in 0..(size.y as usize - 1) {
                    sum += 0.5
                        * (conditional_cdf[conditional_off + (y + 1) * size.x as usize - 1] as f64
                            + conditional_cdf[conditional_off + (y + 2) * size.x as usize - 1]
                                as f64);
                    marginal_cdf[marginal_off + y + 1] = sum as f32;
                }

                let normalization = 1.0 / marginal_cdf[marginal_off + size.y as usize - 1];
                for i in 0..n_values {
                    conditional_cdf[conditional_off + i] *= normalization;
                }
                for i in 0..size.y as usize {
                    marginal_cdf[marginal_off + i] *= normalization;
                }
                for i in 0..n_values {
                    data_out[data_out_off + i] = data[data_off + i] * normalization;
                }

                marginal_off += size.y as usize;
                conditional_off += n_values;
                data_out_off += n_values;
                data_off += n_values;
            }

            (marginal_cdf, conditional_cdf)
        } else {
            data_out.clone_from_slice(data);

            for _slice in 0..slices {
                let mut normalization = 1.0 / (inv_patch_size.x * inv_patch_size.y);
                if normalize {
                    let mut sum = 0.0;
                    for y in 0..size.y as usize - 1 {
                        let mut i = y * size.x as usize;
                        for _x in 0..(size.x as usize - 1) {
                            let v00 = data[data_off + i];
                            let v10 = data[data_off + i + 1];
                            let v01 = data[data_off + i + size.x as usize];
                            let v11 = data[data_off + i + 1 + size.x as usize];
                            let avg = 0.25 * (v00 + v10 + v01 + v11);
                            sum += avg as f64;
                            i += 1;
                        }
                    }
                    normalization = 1.0 / sum as f32;
                }
                for k in 0..n_values {
                    data_out[data_out_off + k] = data[data_off + k] * normalization;
                }

                data_out_off += n_values;
                data_off += n_values;
            }

            (vec![], vec![])
        };

        STATS.bsdfs_created.inc();

        Ok(Self {
            size,
            patch_size: Vector2::splat(1.0) / (size.as_vec2() - Vector2::splat(1.0)),
            inv_patch_size,
            param_size,
            param_values,
            param_strides,
            data: data_out,
            marginal_cdf,
            conditional_cdf,
        })
    }

    fn lookup(&self, d: usize, data: &[f32], i0: usize, size: usize, param_weight: &[f32]) -> f32 {
        if d != 0 {
            let i1 = i0 + self.param_strides[d - 1] * size;
            let w0 = param_weight[2 * d - 2];
            let w1 = param_weight[2 * d - 1];

            let v0 = self.lookup(d - 1, data, i0, size, param_weight);
            let v1 = self.lookup(d - 1, data, i1, size, param_weight);
            v0.mul_add(w0, v1 * w1)
        } else {
            data[i0]
        }
    }

    pub fn sample(&self, mut sample: Vector2, param: &[f32]) -> (Vector2, f32) {
        // Avoid degeneracies at the extrema
        sample = sample.clamp(Vector2::splat(1.0 - 0.99999994), Vector2::splat(0.99999994));

        let mut param_weight = [0.0; 2 * N];
        let mut slice_offset = 0;
        let mut v = Vector2::ZERO;
        for dim in 0..N {
            if self.param_size[dim] == 1 {
                param_weight[2 * dim] = 1.0;
                param_weight[2 * dim + 1] = 0.0;
                continue;
            }

            let param_index = find_interval(self.param_size[dim], |idx| {
                self.param_values[dim][idx] <= param[dim]
            });

            let p0 = self.param_values[dim][param_index];
            let p1 = self.param_values[dim][param_index + 1];

            param_weight[2 * dim + 1] = ((param[dim] - p0) / (p1 - p0)).clamp(0.0, 1.0);
            param_weight[2 * dim] = 1.0 - param_weight[2 * dim + 1];
            slice_offset += self.param_strides[dim] * param_index;
            v[dim] = param_index as f32;
        }

        let mut offset = if N == 0 {
            0
        } else {
            slice_offset * self.size.y as usize
        };

        let fetch_marginal = |idx: usize| {
            self.lookup(
                N,
                &self.marginal_cdf,
                offset + idx,
                self.size.y as usize,
                &param_weight,
            )
        };

        let row = find_interval(self.size.y as usize, |idx| fetch_marginal(idx) < sample.y);
        sample.y -= fetch_marginal(row);

        let slice_size = (self.size.x * self.size.y) as usize;
        offset = row * self.size.x as usize;

        if N != 0 {
            offset += slice_offset * slice_size
        }

        let r0 = self.lookup(
            N,
            &self.conditional_cdf,
            offset + self.size.x as usize - 1,
            slice_size,
            &param_weight,
        );

        let r1 = self.lookup(
            N,
            &self.conditional_cdf,
            offset + self.size.x as usize * 2 - 1,
            slice_size,
            &param_weight,
        );

        let is_const = (r0 - r1).abs() < 0.0001 * (r0 + r1);
        sample.y = if is_const {
            (2.0 * sample.y) / (r0 + r1)
        } else {
            (r0 - (r0 * r0 - 2.0 * sample.y * (r0 - r1)).sqrt()) / (r0 - r1)
        };

        sample.x *= (1.0 - sample.y) * r0 + sample.y * r1;

        let fetch_conditional = |idx| {
            let v0 = self.lookup(
                N,
                &self.conditional_cdf,
                offset + idx,
                slice_size,
                &param_weight,
            );
            let v1 = self.lookup(
                N,
                &self.conditional_cdf[self.size.x as usize..],
                offset + idx,
                slice_size,
                &param_weight,
            );

            (1.0 - sample.y) * v0 + sample.y * v1
        };

        let col = find_interval(self.size.x as usize, |idx| {
            fetch_conditional(idx) < sample.x
        });

        sample.x -= fetch_conditional(col);

        offset += col;

        let v00 = self.lookup(N, &self.data, offset, slice_size, &param_weight);
        let v10 = self.lookup(N, &self.data[1..], offset, slice_size, &param_weight);
        let v01 = self.lookup(
            N,
            &self.data[self.size.x as usize..],
            offset,
            slice_size,
            &param_weight,
        );
        let v11 = self.lookup(
            N,
            &self.data[self.size.x as usize + 1..],
            offset,
            slice_size,
            &param_weight,
        );
        let c0 = (1.0 - sample.y).mul_add(v00, sample.y * v01);
        let c1 = (1.0 - sample.y).mul_add(v10, sample.y * v11);
        let is_const = (c0 - c1).abs() < 1e-4 * (c0 + c1);
        sample.x = if is_const {
            (2.0 * sample.x) / (c0 + c1)
        } else {
            (c0 - (c0 * c0 - 2.0 * sample.x * (c0 - c1)).sqrt()) / (c0 - c1)
        };

        (
            (Vector2::new(col as f32, row as f32) + sample) * self.patch_size,
            ((1.0 - sample.x) * c0 + sample.x * c1)
                * (self.inv_patch_size.x * self.inv_patch_size.y),
        )
    }

    pub fn eval(&self, mut pos: Vector2, param: &[f32]) -> f32 {
        let mut param_weight = [0.0; 2 * N];
        let mut slice_offset = 0;

        for d in 0..N {
            if self.param_size[d] == 1 {
                param_weight[2 * d] = 1.0;
                param_weight[2 * d + 1] = 0.0;
                continue;
            }

            let param_index = find_interval(self.param_size[d], |idx| {
                self.param_values[d][idx] <= param[d]
            });

            let p0 = self.param_values[d][param_index];
            let p1 = self.param_values[d][param_index + 1];

            param_weight[2 * d + 1] = ((param[d] - p0) / (p1 - p0)).clamp(0.0, 1.0);
            param_weight[2 * d] = 1.0 - param_weight[2 * d + 1];
            slice_offset += self.param_strides[d] * param_index;
        }

        pos *= self.inv_patch_size;
        let offset = pos.min(self.size.as_vec2() - Vector2::splat(2.0)).floor();

        let w1 = pos - offset;
        let w0 = Vector2::splat(1.0) - w1;

        let size = (self.size.x * self.size.y) as usize;
        let index = (offset.x + offset.y * self.size.x as f32) as usize
            + if N != 0 { slice_offset * size } else { 0 };

        let v00 = self.lookup(N, &self.data, index, size, &param_weight);
        let v10 = self.lookup(N, &self.data[1..], index, size, &param_weight);
        let v01 = self.lookup(
            N,
            &self.data[self.size.x as usize..],
            index,
            size,
            &param_weight,
        );
        let v11 = self.lookup(
            N,
            &self.data[self.size.x as usize + 1..],
            index,
            size,
            &param_weight,
        );

        w0.y.mul_add(
            w0.x.mul_add(v00, w1.x * v10),
            w1.y * w0.x.mul_add(v01, w1.x * v11),
        ) * (self.inv_patch_size.x * self.inv_patch_size.y)
    }

    pub fn invert(&self, mut sample: Vector2, param: &[f32]) -> (Vector2, f32) {
        let mut param_weight = [0.0; 2 * N];
        let mut slice_offset = 0;
        for d in 0..N {
            if self.param_size[d] == 1 {
                param_weight[2 * d] = 1.0;
                param_weight[2 * d + 1] = 0.0;
                continue;
            }

            let param_index = find_interval(self.param_size[d], |idx| {
                self.param_values[d][idx] <= param[d]
            });

            let p0 = self.param_values[d][param_index];
            let p1 = self.param_values[d][param_index + 1];

            param_weight[2 * d + 1] = ((param[d] - p0) / (p1 - p0)).clamp(0.0, 1.0);
            param_weight[2 * d] = 1.0 - param_weight[2 * d + 1];
            slice_offset += self.param_strides[d] * param_index;
        }

        // Fetch values at corners of bilinear patch
        sample *= self.inv_patch_size;
        let pos = sample.as_uvec2().min(self.size - UVector2::splat(2));
        sample -= pos.as_vec2();

        let mut offset = (pos.x + pos.y * self.size.x) as usize;
        let slice_size = (self.size.x * self.size.y) as usize;
        if N != 0 {
            offset += slice_offset * slice_size;
        }

        /* Invert the X component */
        let v00 = self.lookup(N, &self.data, offset, slice_size, &param_weight);
        let v10 = self.lookup(N, &self.data[1..], offset, slice_size, &param_weight);
        let v01 = self.lookup(
            N,
            &self.data[self.size.x as usize..],
            offset,
            slice_size,
            &param_weight,
        );
        let v11 = self.lookup(
            N,
            &self.data[self.size.x as usize + 1..],
            offset,
            slice_size,
            &param_weight,
        );

        let w1 = sample;
        let w0 = Vector2::splat(1.0) - w1;

        let c0 = w0.y.mul_add(v00, w1.y * v01);
        let c1 = w0.y.mul_add(v10, w1.y * v11);
        let pdf = w0.x.mul_add(c0, w1.x * c1);

        sample.x *= c0 + 0.5 * sample.x * (c1 - c0);

        let v0 = self.lookup(N, &self.conditional_cdf, offset, slice_size, &param_weight);
        let v1 = self.lookup(
            N,
            &self.conditional_cdf[self.size.x as usize..],
            offset,
            slice_size,
            &param_weight,
        );

        sample.x += (1.0 - sample.y) * v0 + sample.y * v1;

        offset = (pos.y * self.size.x) as usize;
        if N != 0 {
            offset += slice_offset * slice_size;
        }

        let r0 = self.lookup(
            N,
            &self.conditional_cdf,
            offset + self.size.x as usize - 1,
            slice_size,
            &param_weight,
        );
        let r1 = self.lookup(
            N,
            &self.conditional_cdf,
            offset + (self.size.x as usize * 2 - 1),
            slice_size,
            &param_weight,
        );

        sample.x /= (1.0 - sample.y) * r0 + sample.y * r1;

        /* Invert the Y component */
        sample.y *= r0 + 0.5 * sample.y * (r1 - r0);

        offset = pos.y as usize;
        if N != 0 {
            offset += slice_offset * self.size.y as usize;
        }

        sample.y += self.lookup(
            N,
            &self.marginal_cdf,
            offset,
            self.size.y as usize,
            &param_weight,
        );

        (sample, pdf * self.inv_patch_size.x * self.inv_patch_size.y)
    }
}

pub type Warp2D0 = Marginal2d<0>;
pub type Warp2D2 = Marginal2d<2>;
pub type Warp2D3 = Marginal2d<3>;

#[derive(Debug, PartialEq)]

enum TensorFieldType {
    Invalid,

    UInt8,
    Int8,
    UInt16,
    Int16,
    UInt32,
    Int32,
    UInt64,
    Int64,

    Float16,
    Float32,
    Float64,
}

impl From<u8> for TensorFieldType {
    fn from(x: u8) -> Self {
        match x {
            1 => TensorFieldType::UInt8,
            2 => TensorFieldType::Int8,
            3 => TensorFieldType::UInt16,
            4 => TensorFieldType::Int16,
            5 => TensorFieldType::UInt32,
            6 => TensorFieldType::Int32,
            7 => TensorFieldType::UInt64,
            8 => TensorFieldType::Int64,
            9 => TensorFieldType::Float16,
            10 => TensorFieldType::Float32,
            11 => TensorFieldType::Float64,
            _ => TensorFieldType::Invalid,
        }
    }
}

impl TensorFieldType {
    pub fn size(&self) -> usize {
        match self {
            TensorFieldType::Invalid => 0,
            TensorFieldType::UInt8 => 1,
            TensorFieldType::Int8 => 1,
            TensorFieldType::UInt16 => 2,
            TensorFieldType::Int16 => 2,
            TensorFieldType::UInt32 => 4,
            TensorFieldType::Int32 => 4,
            TensorFieldType::UInt64 => 8,
            TensorFieldType::Int64 => 8,
            TensorFieldType::Float16 => 2,
            TensorFieldType::Float32 => 4,
            TensorFieldType::Float64 => 8,
        }
    }
}

#[derive(Debug)]
struct TensorField {
    dtype: TensorFieldType,
    shape: Vec<usize>,
    data: Vec<u8>,
}

#[derive(Debug)]
struct TensorFile {
    pub fields: HashMap<String, TensorField>,
}

impl TensorFile {
    pub fn load_from_file(path: &Path) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let size = file.metadata()?.len() as usize;
        let mut reader = BufReader::new(file);
        let mut fields = HashMap::default();

        if size < 12 + 2 + 4 {
            Err("Invalid tensor file: too small :<")?;
        }
        let mut header = [0; 12];
        reader.read_exact(&mut header)?;
        // utf8 is equal to ascii here
        if std::str::from_utf8(&header)? != "tensor_file\0" {
            Err("Invalid tensor file: bad header ;;")?;
        }
        let mut version = [0; 2];
        reader.read_exact(&mut version)?;
        if version[0] != 1 || version[1] != 0 {
            Err("Invalid tensor file: unknown version :(")?;
        }

        let n_fields = {
            let mut bytes = [0; 4];
            reader.read_exact(&mut bytes)?;
            u32::from_le_bytes(bytes)
        };

        for _ in 0..n_fields {
            let name_len = {
                let mut data = [0; 2];
                reader.read_exact(&mut data)?;
                u16::from_le_bytes(data)
            } as usize;
            let name = {
                let mut data = vec![0; name_len];
                reader.read_exact(&mut data)?;
                String::from_utf8(data)?
            };

            let ndim = {
                let mut data = [0; 2];
                reader.read_exact(&mut data)?;
                u16::from_le_bytes(data)
            } as usize;

            let dtype = {
                let mut data = [0; 1];
                reader.read_exact(&mut data)?;
                let dtype = TensorFieldType::from(data[0]);
                if dtype == TensorFieldType::Invalid {
                    Err(format!(
                        "Invalid tensor file: unknown datatype in {name} ;;"
                    ))?
                }
                dtype
            };

            let offset = {
                let mut data = [0; 8];
                reader.read_exact(&mut data)?;
                u64::from_le_bytes(data)
            } as usize;

            let mut shape = Vec::with_capacity(ndim);
            let mut total_size = dtype.size();
            for i in 0..ndim {
                let sizev = {
                    let mut data = [0; 8];
                    reader.read_exact(&mut data)?;
                    u64::from_le_bytes(data)
                } as usize;
                shape.push(sizev);
                total_size *= shape[i];
            }
            let pos = reader.stream_position()?;
            let mut data = vec![0; total_size];
            reader.seek(SeekFrom::Start(offset as u64))?;
            reader.read_exact(&mut data)?;

            reader.seek(SeekFrom::Start(pos))?;
            fields.insert(name, TensorField { dtype, shape, data });
        }

        Ok(TensorFile { fields })
    }
}

fn u2theta(u: f32) -> f32 {
    u * u * (std::f32::consts::PI / 2.0)
}

fn u2phi(u: f32) -> f32 {
    (2.0 * u - 1.0) * std::f32::consts::PI
}

fn theta2u(theta: f32) -> f32 {
    (theta * (2.0 / std::f32::consts::PI)).sqrt()
}

fn phi2u(phi: f32) -> f32 {
    (phi + std::f32::consts::PI) / std::f32::consts::TAU
}

#[derive(Debug, Clone)]
struct MeasuredBrdfData {
    pub ndf: Warp2D0,
    pub sigma: Warp2D0,
    pub vndf: Warp2D2,
    pub luminance: Warp2D2,
    pub rgb: Warp2D3,
    pub isotropic: bool,
    pub jacobian: bool,
}

#[derive(Debug, Clone)]
pub struct MeasuredBsdf {
    //TODO: this should be in a centralized store for coherency reasons
    //NOTE: this is boxed so the size of Bsdf is smaller, maybe it will become unboxed if other bsdfs are larger.
    data: Box<MeasuredBrdfData>,
}

impl MeasuredBsdf {
    pub fn load_from_file(path: &Path) -> Result<Self, Box<dyn Error>> {
        let tf = TensorFile::load_from_file(path)?;
        let theta_i = tf
            .fields
            .get("theta_i")
            .ok_or("bad tensor file, no theta_i")?;
        let phi_i = tf.fields.get("phi_i").ok_or("bad tensor file, no phi_i")?;
        let ndf = tf.fields.get("ndf").ok_or("bad tensor file, no ndf")?;
        let sigma = tf.fields.get("sigma").ok_or("bad tensor file, no sigma")?;
        let vndf = tf.fields.get("vndf").ok_or("bad tensor file, no vndf")?;
        let rgb = tf.fields.get("rgb").ok_or("bad tensor file, no rgb")?;
        let luminance = tf
            .fields
            .get("luminance")
            .ok_or("bad tensor file, no luminance")?;
        let description = tf
            .fields
            .get("description")
            .ok_or("bad tensor file, no description")?;
        let jacobian = tf
            .fields
            .get("jacobian")
            .ok_or("bad tensor file, no jacobian")?;

        // handle errors.
        {
            if !(description.shape.len() == 1 && description.dtype == TensorFieldType::UInt8) {
                Err("Invalid tensor file: bad description.")?;
            }
            if !(theta_i.shape.len() == 1 && theta_i.dtype == TensorFieldType::Float32) {
                Err("Invalid tensor file: bad theta_i.")?;
            }
            if !(phi_i.shape.len() == 1 && phi_i.dtype == TensorFieldType::Float32) {
                Err("Invalid tensor file: bad phi_i.")?;
            }
            if !(ndf.shape.len() == 2 && ndf.dtype == TensorFieldType::Float32) {
                Err("Invalid tensor file: bad ndf.")?;
            }
            if !(sigma.shape.len() == 2 && sigma.dtype == TensorFieldType::Float32) {
                Err("Invalid tensor file: bad sigma.")?;
            }
            if !(vndf.shape.len() == 4
                && vndf.dtype == TensorFieldType::Float32
                && vndf.shape[0] == phi_i.shape[0]
                && vndf.shape[1] == theta_i.shape[0])
            {
                Err("Invalid tensor file: bad vndf.")?;
            }
            if !(luminance.shape.len() == 4
                && luminance.dtype == TensorFieldType::Float32
                && luminance.shape[0] == phi_i.shape[0]
                && luminance.shape[1] == theta_i.shape[0]
                && luminance.shape[2] == luminance.shape[3])
                && luminance.shape[3] == rgb.shape[4]
            {
                Err("Invalid tensor file: bad luminance.")?;
            }
            if !(rgb.dtype == TensorFieldType::Float32
                && rgb.shape.len() == 5
                && rgb.shape[0] == phi_i.shape[0]
                && rgb.shape[1] == theta_i.shape[0]
                && rgb.shape[2] == 3
                && rgb.shape[3] == luminance.shape[2])
            {
                Err("Invalid tensor file: bad rgb.")?;
            }
            if !(jacobian.shape.len() == 1
                && jacobian.shape[0] == 1
                && jacobian.dtype == TensorFieldType::UInt8)
            {
                Err("Invalid tensor file: bad jacobian.")?;
            }
        }

        let isotropic = phi_i.shape[0] <= 2;
        let jacobian = jacobian.data[0] != 0;

        let phi_i_data = phi_i
            .data
            .chunks(4)
            .map(|x| f32::from_le_bytes(x.try_into().expect("oop")))
            .collect::<Vec<_>>();

        let theta_i_data = theta_i
            .data
            .chunks(4)
            .map(|x| f32::from_le_bytes(x.try_into().expect("oop")))
            .collect::<Vec<_>>();

        if !isotropic
            && (std::f32::consts::TAU / (phi_i_data[phi_i.shape[0] - 1] - phi_i_data[0])) != 1.0
        {
            Err("Reduction != 1.0 not supported")?;
        }

        let ndf = Warp2D0::new(
            UVector2::new(ndf.shape[1] as u32, ndf.shape[0] as u32),
            &ndf.data
                .chunks(4)
                .map(|x| f32::from_le_bytes(x.try_into().expect("oop")))
                .collect::<Vec<_>>(),
            [],
            false,
            false,
        )?;

        let sigma = Warp2D0::new(
            UVector2::new(sigma.shape[1] as u32, sigma.shape[0] as u32),
            &sigma
                .data
                .chunks(4)
                .map(|x| f32::from_le_bytes(x.try_into().expect("oop")))
                .collect::<Vec<_>>(),
            [],
            false,
            false,
        )?;

        let vndf = Warp2D2::new(
            UVector2::new(vndf.shape[3] as u32, vndf.shape[2] as u32),
            &vndf
                .data
                .chunks(4)
                .map(|x| f32::from_le_bytes(x.try_into().expect("oop")))
                .collect::<Vec<_>>(),
            [phi_i_data.clone(), theta_i_data.clone()],
            true,
            true,
        )?;

        let luminance = Warp2D2::new(
            UVector2::new(luminance.shape[3] as u32, luminance.shape[2] as u32),
            &luminance
                .data
                .chunks(4)
                .map(|x| f32::from_le_bytes(x.try_into().expect("oop")))
                .collect::<Vec<_>>(),
            [phi_i_data.clone(), theta_i_data.clone()],
            true,
            true,
        )?;

        let channels = vec![0.0, 1.0, 2.0];
        let rgb = Warp2D3::new(
            UVector2::new(rgb.shape[4] as u32, rgb.shape[3] as u32),
            &rgb.data
                .chunks(4)
                .map(|x| f32::from_le_bytes(x.try_into().expect("oop")))
                .collect::<Vec<_>>(),
            [phi_i_data, theta_i_data, channels],
            false,
            false,
        )?;

        Ok(MeasuredBsdf {
            data: Box::new(MeasuredBrdfData {
                ndf,
                sigma,
                vndf,
                luminance,
                rgb,
                isotropic,
                jacobian,
            }),
        })
    }
}

impl BsdfT for MeasuredBsdf {
    fn sample(&self, wi: Vector3, _si: &SurfaceInteraction, u: Point2) -> BsdfSample {
        // return BsdfSample {
        //     wo: wi,
        //     sampled: BsdfFlags::Delta,
        //     spectrum: Spectrum::from_rgb(wi.x, wi.y, wi.z),
        // };

        let (wi, flip_wo) = if wi.z <= 0.0 {
            (-wi, true)
        } else {
            (wi, false)
        };

        let theta_i = spherical_theta(wi);
        let phi_i = wi.y.atan2(wi.x);

        let params = [phi_i, theta_i];
        let sample = Vector2::new(u.y, u.x);
        // let lum_pdf = 1.0;

        let (sample, lum_pdf) = self.data.luminance.sample(sample, &params);

        let (u_wm, ndf_pdf) = self.data.vndf.sample(sample, &params);

        let phi_m = u2phi(u_wm.y) + if self.data.isotropic { phi_i } else { 0.0 };
        let theta_m = u2theta(u_wm.x);

        /* Spherical -> Cartesian coordinates */
        let (sin_phi_m, cos_phi_m) = phi_m.sin_cos();
        let (sin_theta_m, cos_theta_m) = theta_m.sin_cos();

        let wm = Vector3::new(
            cos_phi_m * sin_theta_m,
            sin_phi_m * sin_theta_m,
            cos_theta_m,
        );

        let wo = reflect_across(wi, wm);

        if wo.z <= 0.0 {
            return BsdfSample {
                wo: Vector3::Z,
                sampled: self.flags(),
                spectrum: Spectrum::zero(),
            };
        }

        let wo = wo * if flip_wo { -1.0 } else { 1.0 };

        let mut fr = Vector3::ZERO;
        for i in 0..3 {
            let params_fr = [phi_i, theta_i, i as f32];

            fr[i] = self.data.rgb.eval(sample, &params_fr).max(0.0);
        }

        let u_wi = Vector2::new(theta2u(theta_i), phi2u(phi_i));
        fr *= self.data.ndf.eval(u_wm, &params) / (4.0 * self.data.sigma.eval(u_wi, &params));

        let jacobian = (2.0 * core::f32::consts::PI.powi(2) * u_wm.x * sin_theta_m).max(1e-6)
            * 4.0
            * wi.dot(wm);

        let pdf = ndf_pdf * lum_pdf / jacobian;

        BsdfSample {
            wo,
            sampled: self.flags(),
            spectrum: Spectrum::from_rgb(fr.x, fr.y, fr.z) / pdf,
        }
    }

    fn eval(&self, _si: &SurfaceInteraction, mut wi: Vector3, mut wo: Vector3) -> Spectrum {
        if wo.z * wi.z < 0.0 {
            return Spectrum::zero();
        }

        if wo.z < 0.0 {
            wo = -wo;
            wi = -wi;
        }

        let wm = wi + wo;
        if wm.length_squared() == 0.0 {
            return Spectrum::zero();
        }
        let wm = wm.normalize();

        /* Cartesian -> spherical coordinates */
        let theta_i = spherical_theta(wi);
        let phi_i = wi.y.atan2(wi.x);
        let theta_m = spherical_theta(wm);
        let phi_m = wm.y.atan2(wm.x);

        /* Spherical coordinates -> unit coordinate system */
        let u_wi = Vector2::new(theta2u(theta_i), phi2u(phi_i));
        let mut u_wm = Vector2::new(
            theta2u(theta_m),
            phi2u(if self.data.isotropic {
                phi_m - phi_i
            } else {
                phi_m
            }),
        );
        u_wm.y = u_wm.y.fract();

        let params = [phi_i, theta_i];
        let (sample, _vndf_pdf) = self.data.vndf.invert(u_wm, &params);

        let mut fr = Vector3::ZERO;
        for i in 0..3 {
            let params_fr = [phi_i, theta_i, i as f32];

            fr[i] = self.data.rgb.eval(sample, &params_fr);

            /* clamp the value to zero (negative values occur when the original
            spectral data goes out of gamut) */
            fr[i] = fr[i].max(0.0);
        }

        fr *= self.data.ndf.eval(u_wm, &params) / (4.0 * self.data.sigma.eval(u_wi, &params));

        Spectrum::from_rgb(fr.x, fr.y, fr.z)
    }

    fn flags(&self) -> BsdfFlags {
        BsdfFlags::DiffuseReflection
        // BsdfFlags::DeltaReflection
    }
}

#[cfg(test)]
mod tests {
    use crate::{media::MediumInterface, shapes::Sphere};

    use super::*;

    #[test]
    fn scale() -> Result<(), Box<dyn Error>> {
        let brdf = MeasuredBsdf::load_from_file(Path::new(
            "/Users/adenine/Developer/luminiferous/assets/brdfs/vch_golden_yellow_rgb.bsdf",
        ))?;
        let size = 5;
        let primitive = crate::primitive::Primitive::new(
            crate::shapes::Shape::Sphere(Sphere::new(0.5)),
            0,
            None,
            None,
            MediumInterface::none(),
        );
        let si = &SurfaceInteraction {
            primitive: &primitive,
            t: 0.0,
            p: Point3::default(),
            n: Normal3::default(),
            uv: Point2::default(),
            dp_du: Normal3::default(),
            dp_dv: Normal3::default(),
        };
        println!("{{ \"data\": [");
        for wix in -size..size {
            for wiy in -size..size {
                for wiz in 0..size {
                    for wox in -size..size {
                        for woy in -size..size {
                            for woz in 0..size {
                                let wi = (Vector3::new(
                                    wix as f32 / size as f32,
                                    wiy as f32 / size as f32,
                                    wiz as f32 / size as f32,
                                ))
                                .normalize();
                                let wo = (Vector3::new(
                                    wox as f32 / size as f32,
                                    woy as f32 / size as f32,
                                    woz as f32 / size as f32,
                                ))
                                .normalize();

                                let res = brdf.eval(si, wi, wo);
                                println!("{{");
                                {
                                    println!("\t\"wi.x\": {},", wi.x);
                                    println!("\t\"wi.y\": {},", wi.y);
                                    println!("\t\"wi.z\": {},", wi.z);

                                    println!("\t\"wo.x\": {},", wi.x);
                                    println!("\t\"wo.y\": {},", wi.y);
                                    println!("\t\"wo.z\": {},", wi.z);

                                    println!("\t\"res.r\": {},", res[0]);
                                    println!("\t\"res.g\": {},", res[1]);
                                    println!("\t\"res.b\": {},", res[2]);
                                }
                                println!("}},");
                            }
                        }
                    }
                }
            }
        }
        println!("] }}");

        assert!(false);
        Ok(())
    }
}
