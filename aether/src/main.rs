use luminiferous::{Context, ContextParams};

fn main() {
    //TODO: cmd line parsing
    Context::new(ContextParams { seed: 0, spp: 2048 }).run();
}
