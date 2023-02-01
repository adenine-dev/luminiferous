use luminiferous as lum;

use tempdir::TempDir;

#[test]
fn cpu_gpu_consistent() {
    let dir = TempDir::new("cpu_gpu_consistency_test").unwrap();

    lum::run(lum::Config {
        width: 480,
        height: 360,
        gpu: false,
        output_path: dir.path().join("cpu.exr").into(),
    })
    .unwrap();

    lum::run(lum::Config {
        width: 480,
        height: 360,
        gpu: true,
        output_path: dir.path().join("gpu.exr").into(),
    })
    .unwrap();

    use image::io::Reader as ImageReader;
    let cpu_image = ImageReader::open(dir.path().join("cpu.exr"))
        .unwrap()
        .decode()
        .unwrap();
    let cpu_image_data = cpu_image.as_rgba32f().unwrap();

    let gpu_image = ImageReader::open(dir.path().join("gpu.exr"))
        .unwrap()
        .decode()
        .unwrap();
    let gpu_image_data = gpu_image.as_rgba32f().unwrap();

    assert_eq!(cpu_image_data.len(), gpu_image_data.len());

    cpu_image_data
        .iter()
        .zip(gpu_image_data.iter())
        .for_each(|(cpu, gpu)| assert!((cpu - gpu).abs() < f32::EPSILON));
}
