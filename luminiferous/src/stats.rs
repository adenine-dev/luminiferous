use std::{
    sync::atomic::{AtomicI64, AtomicU64, Ordering},
    time::Duration,
};

pub struct StatCounter {
    pub count: AtomicU64,
}

impl StatCounter {
    pub const fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
        }
    }

    pub fn inc(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add(&self, n: u64) {
        self.count.fetch_add(n, Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.count.load(Ordering::SeqCst)
    }

    pub fn get_as_bytes(&self) -> String {
        // assumes nothing goes above tb ig
        let units: [&str; 4] = ["B", "KB", "MiB", "GiB"];

        let mut size = self.get() as f32;

        for unit in units {
            if size < 1024.0 {
                return format!("{:.2} {}", size, unit);
            } else {
                size /= 1024.0
            }
        }

        format!("{:.2} TiB", size)
    }

    pub fn get_as_duration(&self) -> String {
        let duration = Duration::from_millis(self.get());

        format!("{duration:?}")
    }
}

pub struct StatIntDistribution {
    pub count: AtomicU64,
    pub min: AtomicI64,
    pub max: AtomicI64,
    pub sum: AtomicI64,
}

impl StatIntDistribution {
    pub const fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
            min: AtomicI64::new(i64::MAX),
            max: AtomicI64::new(i64::MIN),
            sum: AtomicI64::new(0),
        }
    }

    pub fn add(&self, n: i64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.sum.fetch_add(n, Ordering::Relaxed);
        self.min.fetch_min(n, Ordering::Relaxed);
        self.max.fetch_max(n, Ordering::Relaxed);
    }

    pub fn get_avg(&self) -> f32 {
        self.sum.load(Ordering::SeqCst) as f32 / self.count.load(Ordering::SeqCst) as f32
    }

    pub fn get_min(&self) -> i64 {
        self.min.load(Ordering::SeqCst)
    }

    pub fn get_max(&self) -> i64 {
        self.max.load(Ordering::SeqCst)
    }

    pub fn get_count(&self) -> u64 {
        self.count.load(Ordering::SeqCst)
    }
}

pub struct Statistics {
    // timing things
    pub init_duration: StatCounter,
    pub render_duration: StatCounter,

    // scene things
    pub shapes_created: StatCounter,
    pub bsdfs_created: StatCounter,
    pub materials_created: StatCounter,
    pub lights_created: StatCounter,
    pub textures_created: StatCounter,

    // rendery things
    pub camera_rays_traced: StatCounter,
    pub zero_radiance_paths: StatCounter,
    pub path_length: StatIntDistribution,
    pub regular_intersection_tests: StatCounter,
    pub shadow_intersection_tests: StatCounter,

    // memory things
    pub texture_memory: StatCounter,
    pub film_memory: StatCounter,
    pub primitive_memory: StatCounter,
}

impl Statistics {
    pub const fn new() -> Self {
        Self {
            init_duration: StatCounter::new(),
            render_duration: StatCounter::new(),

            shapes_created: StatCounter::new(),
            bsdfs_created: StatCounter::new(),
            materials_created: StatCounter::new(),
            lights_created: StatCounter::new(),
            textures_created: StatCounter::new(),

            camera_rays_traced: StatCounter::new(),
            zero_radiance_paths: StatCounter::new(),
            path_length: StatIntDistribution::new(),

            regular_intersection_tests: StatCounter::new(),
            shadow_intersection_tests: StatCounter::new(),

            texture_memory: StatCounter::new(),
            film_memory: StatCounter::new(),
            primitive_memory: StatCounter::new(),
        }
    }

    #[rustfmt::skip] // sorry people with not wide monitors :<
    pub fn print(&self) {
        println!("stats:");
        let total_duration = (self.init_duration.get() + self.render_duration.get()) as f32;

        println!("  duration:");
        println!("    initialization: {} ({:.2}%)", self.init_duration.get_as_duration(), self.init_duration.get() as f32 / total_duration * 100.0);
        println!("    rendering:      {} ({:.2}%)", self.render_duration.get_as_duration(), self.render_duration.get() as f32 / total_duration * 100.0);

        println!("  scene:");
        println!("    shapes created:    {}", self.shapes_created.get());
        println!("    bsdfs created:     {}", self.bsdfs_created.get());
        println!("    materials created: {}", self.materials_created.get());
        println!("    lights created:    {}", self.lights_created.get());
        println!("    textures created:  {}", self.textures_created.get());

        println!("  render:");
        println!("    camera rays traced:         {}", self.camera_rays_traced.get());
        println!("    zero radiance paths:        {} / {} ({:.2}%)", self.zero_radiance_paths.get(), self.camera_rays_traced.get(), 100.0 * (self.zero_radiance_paths.get() as f32 / self.camera_rays_traced.get() as f32));
        println!("    regular intersection tests: {}", self.regular_intersection_tests.get());
        println!("    shadow intersection tests:  {}", self.shadow_intersection_tests.get());
        println!("    average path length:        {:.2} ({}-{})", self.path_length.get_avg(), self.path_length.get_min(), self.path_length.get_max());

        println!("  Memory:");
        let total_memory = (self.texture_memory.get()
            + self.film_memory.get()
            + self.primitive_memory.get()) as f32;
        println!("    textures:   {} ({:.2}%)", self.texture_memory.get_as_bytes(), self.texture_memory.get() as f32 / total_memory * 100.0);
        println!("    film:       {} ({:.2}%)", self.film_memory.get_as_bytes(), self.film_memory.get() as f32 / total_memory * 100.0);
        println!("    primitives: {} ({:.2}%)", self.primitive_memory.get_as_bytes(), self.primitive_memory.get() as f32 / total_memory * 100.0);
    }
}

pub(crate) static STATS: Statistics = Statistics::new();
